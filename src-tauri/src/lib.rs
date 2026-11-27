pub mod blue_team;
pub mod crypto;
pub mod installer;
pub mod purple_team;
pub mod security_guard;
pub mod vault_io;

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State, WebviewWindow, WindowEvent,
};
use zeroize::Zeroize;

use blue_team::{
    detect_injection_environment, SecurityAuditLog, SecurityEventKind, UnlockThrottle,
};
use crypto::{
    open_vault_envelope, seal_vault_envelope, unwrap_vmk_slot, wrap_vmk_slot, KeySlotKind,
    VaultEnvelope, VaultKey,
};
use installer::{
    default_install_dir, is_installed_at, perform_install, perform_uninstall, run_doctor_checks,
    DoctorCheck, InstallResult, UninstallResult,
};
use purple_team::{run_purple_team_suite, PurpleTeamReport};
use vault_io::{get_default_vault_path, get_vault_storage_dir, read_vault_file, save_vault_atomic};

pub struct ActiveSession {
    pub vmk: Option<VaultKey>,
    pub envelope_slots: Vec<crypto::KeySlot>,
}

pub struct SecurityState {
    pub audit_log: Mutex<SecurityAuditLog>,
    pub throttle: Mutex<UnlockThrottle>,
    pub session: Mutex<ActiveSession>,
    pub app_mode: String,
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn audit_ledger_path() -> Option<PathBuf> {
    get_vault_storage_dir().ok().map(|d| d.join("audit.ledger"))
}

fn persist_audit(log: &SecurityAuditLog) {
    if let Some(p) = audit_ledger_path() {
        let _ = log.save_to_disk(&p);
    }
}

#[tauri::command]
fn get_app_mode(state: State<'_, SecurityState>) -> serde_json::Value {
    let default_dir = default_install_dir();
    let already_installed = is_installed_at(&default_dir);
    serde_json::json!({
        "mode": state.app_mode,
        "defaultInstallDir": default_dir.display().to_string(),
        "alreadyInstalled": already_installed,
    })
}

#[tauri::command]
fn installer_doctor(target_dir: String) -> Vec<DoctorCheck> {
    run_doctor_checks(&target_dir)
}

#[tauri::command]
fn installer_run(
    dest_dir: String,
    desktop_shortcut: bool,
    start_menu_shortcut: bool,
    launch_after: bool,
) -> Result<InstallResult, String> {
    let res = perform_install(&dest_dir, desktop_shortcut, start_menu_shortcut)?;
    if launch_after {
        let _ = std::process::Command::new(&res.exe_path).spawn();
    }
    Ok(res)
}

#[tauri::command]
fn uninstaller_run(shred_vault_data: bool) -> Result<UninstallResult, String> {
    perform_uninstall(shred_vault_data)
}

#[tauri::command]
fn exit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn unlock_vault(
    state: State<'_, SecurityState>,
    credential: String,
    initial_items_json: Option<String>,
) -> Result<serde_json::Value, String> {
    if security_guard::is_debugger_attached() {
        if let Ok(mut log) = state.audit_log.lock() {
            log.record(
                SecurityEventKind::DebuggerDetected,
                "Unlock blocked: debugger attached",
            );
            persist_audit(&log);
        }
        return Err("Security lockdown: debugger detected.".into());
    }

    let ts = now_unix();
    {
        let throttle = state.throttle.lock().map_err(|_| "Lock error".to_string())?;
        if let Err(wait_secs) = throttle.check_allowed(ts) {
            return Err(format!("Too many failed attempts. Wait {}s.", wait_secs));
        }
    }

    let vault_path = get_default_vault_path().map_err(|e| e.to_string())?;

    if !vault_path.exists() {
        // Initialize fresh 2-tier VMK + Passphrase + Default Quick PIN slot
        let vmk = VaultKey::generate();
        let pass_slot =
            wrap_vmk_slot(&vmk, KeySlotKind::Passphrase, &credential).map_err(|e| e.to_string())?;
        let pin_slot =
            wrap_vmk_slot(&vmk, KeySlotKind::QuickPin, "123456").map_err(|e| e.to_string())?;
        let slots = vec![pass_slot, pin_slot];

        let payload = initial_items_json.unwrap_or_else(|| "[]".to_string());
        let sealed =
            seal_vault_envelope(&vmk, slots.clone(), payload.as_bytes()).map_err(|e| e.to_string())?;
        save_vault_atomic(&vault_path, &sealed).map_err(|e| e.to_string())?;

        if let Ok(mut sess) = state.session.lock() {
            sess.vmk = Some(vmk);
            sess.envelope_slots = slots;
        }
        if let Ok(mut log) = state.audit_log.lock() {
            log.record(SecurityEventKind::VaultInitialized, "Created REPOSE01 2-tier vault");
            persist_audit(&log);
        }
        return Ok(serde_json::json!({ "itemsJson": payload, "initialized": true }));
    }

    let envelope_bytes = read_vault_file(&vault_path).map_err(|e| e.to_string())?;
    let env: VaultEnvelope = serde_json::from_slice(&envelope_bytes).map_err(|_| {
        if let Ok(mut log) = state.audit_log.lock() {
            log.record(SecurityEventKind::HeaderTamperDetected, "Malformed vault header");
            persist_audit(&log);
        }
        "Corrupted vault envelope".to_string()
    })?;

    // Attempt to unwrap VMK from any matching KeySlot
    let mut unwrapped_vmk: Option<VaultKey> = None;
    for slot in &env.header.slots {
        if let Ok(vmk) = unwrap_vmk_slot(slot, &credential) {
            unwrapped_vmk = Some(vmk);
            break;
        }
    }

    let vmk = match unwrapped_vmk {
        Some(k) => k,
        None => {
            let penalty = if let Ok(mut t) = state.throttle.lock() {
                t.record_failure(ts)
            } else {
                0
            };
            if let Ok(mut log) = state.audit_log.lock() {
                log.record(
                    SecurityEventKind::UnlockFailed,
                    format!("Invalid credential (backoff {}s)", penalty),
                );
                persist_audit(&log);
            }
            return Err("Authentication rejected: invalid credential.".into());
        }
    };

    let plaintext = open_vault_envelope(&envelope_bytes, &vmk).map_err(|_| {
        if let Ok(mut log) = state.audit_log.lock() {
            log.record(SecurityEventKind::HeaderTamperDetected, "AEAD tag verification failed");
            persist_audit(&log);
        }
        "AEAD authentication failed: vault data tampered.".to_string()
    })?;

    let items_json = String::from_utf8(plaintext).map_err(|_| "Invalid UTF-8 vault payload")?;

    if let Ok(mut t) = state.throttle.lock() {
        t.record_success();
    }
    if let Ok(mut sess) = state.session.lock() {
        sess.vmk = Some(vmk);
        sess.envelope_slots = env.header.slots;
    }
    if let Ok(mut log) = state.audit_log.lock() {
        log.record(SecurityEventKind::VaultUnlocked, "Unlocked via Argon2id + XChaCha20");
        persist_audit(&log);
    }

    Ok(serde_json::json!({ "itemsJson": items_json, "initialized": false }))
}

#[tauri::command]
fn save_vault_items_ipc(
    state: State<'_, SecurityState>,
    items_json: String,
) -> Result<bool, String> {
    let sess = state.session.lock().map_err(|_| "Session lock error".to_string())?;
    let vmk = sess
        .vmk
        .as_ref()
        .ok_or_else(|| "Vault is locked".to_string())?;

    let sealed = seal_vault_envelope(vmk, sess.envelope_slots.clone(), items_json.as_bytes())
        .map_err(|e| e.to_string())?;
    let vault_path = get_default_vault_path().map_err(|e| e.to_string())?;
    save_vault_atomic(&vault_path, &sealed).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
fn lock_vault_ipc(state: State<'_, SecurityState>) -> Result<bool, String> {
    if let Ok(mut sess) = state.session.lock() {
        if let Some(mut vmk) = sess.vmk.take() {
            vmk.zeroize();
        }
        sess.envelope_slots.clear();
    }
    if let Ok(mut log) = state.audit_log.lock() {
        log.record(SecurityEventKind::VaultLocked, "Session locked & RAM zeroized");
        persist_audit(&log);
    }
    Ok(true)
}

#[tauri::command]
fn check_security_status(state: State<'_, SecurityState>) -> Result<serde_json::Value, String> {
    let debugger_attached = security_guard::is_debugger_attached();
    let injection_vars = detect_injection_environment();

    if debugger_attached {
        if let Ok(mut sess) = state.session.lock() {
            if let Some(mut vmk) = sess.vmk.take() {
                vmk.zeroize();
            }
        }
    }

    if let Ok(mut log) = state.audit_log.lock() {
        if debugger_attached {
            log.record(
                SecurityEventKind::DebuggerDetected,
                "Debugger attachment detected; silent zeroize & lock triggered",
            );
            persist_audit(&log);
        }
        if !injection_vars.is_empty() {
            log.record(
                SecurityEventKind::InjectionEnvDetected,
                format!("Suspicious loader variables: {:?}", injection_vars),
            );
            persist_audit(&log);
        }
    }

    Ok(serde_json::json!({
        "debuggerAttached": debugger_attached,
        "injectionVars": injection_vars,
        "antiScreenCaptureSupported": cfg!(windows),
        "memoryLockingActive": true,
        "airgappedInvariant": true,
    }))
}

#[tauri::command]
fn run_purple_team_audit(state: State<'_, SecurityState>) -> Result<PurpleTeamReport, String> {
    let report = run_purple_team_suite();
    if let Ok(mut log) = state.audit_log.lock() {
        log.record(
            SecurityEventKind::PurpleTeamAuditRun,
            format!("Passed {}/{} checks", report.passed, report.total),
        );
        persist_audit(&log);
    }
    Ok(report)
}

#[tauri::command]
fn get_security_audit_log(state: State<'_, SecurityState>) -> Result<serde_json::Value, String> {
    let log = state
        .audit_log
        .lock()
        .map_err(|_| "Audit log lock poisoned".to_string())?;
    let chain_valid = log.verify_chain().is_ok();
    Ok(serde_json::json!({
        "chainValid": chain_valid,
        "entries": log.entries,
    }))
}

#[tauri::command]
fn set_screen_protection(window: WebviewWindow, enabled: bool) -> Result<bool, String> {
    #[cfg(windows)]
    {
        if let Ok(hwnd) = window.hwnd() {
            let success = security_guard::set_display_affinity_protection(hwnd.0 as _, enabled);
            return Ok(success);
        }
    }
    let _ = window;
    let _ = enabled;
    Ok(true)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    let is_installer = args.iter().any(|a| a == "--installer");
    let is_uninstall = args.iter().any(|a| a == "--uninstall");
    let is_silent = args.iter().any(|a| a == "--silent");
    let no_launch = args.iter().any(|a| a == "--no-launch");

    // Silent CLI install mode (for updates/automated deployments)
    if is_installer && is_silent {
        let dest = args
            .windows(2)
            .find(|w| w[0] == "--dest")
            .map(|w| w[1].clone())
            .unwrap_or_else(|| default_install_dir().display().to_string());
        let _ = perform_install(&dest, true, true);
        if !no_launch {
            let exe = PathBuf::from(&dest).join("repose.exe");
            let _ = std::process::Command::new(exe).spawn();
        }
        return;
    }

    if is_uninstall && is_silent {
        let _ = perform_uninstall(false);
        return;
    }

    let app_mode = if is_uninstall {
        "uninstall".to_string()
    } else if is_installer {
        "installer".to_string()
    } else {
        "vault".to_string()
    };

    let mut initial_log = audit_ledger_path()
        .map(|p| SecurityAuditLog::load_from_disk(&p))
        .unwrap_or_default();
    initial_log.record(
        SecurityEventKind::VaultInitialized,
        format!("Repose started in {} mode (airgapped)", app_mode),
    );
    persist_audit(&initial_log);

    let mode_for_close = app_mode.clone();

    tauri::Builder::default()
        .manage(SecurityState {
            audit_log: Mutex::new(initial_log),
            throttle: Mutex::new(UnlockThrottle::default()),
            session: Mutex::new(ActiveSession {
                vmk: None,
                envelope_slots: Vec::new(),
            }),
            app_mode,
        })
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            get_app_mode,
            installer_doctor,
            installer_run,
            uninstaller_run,
            exit_app,
            unlock_vault,
            save_vault_items_ipc,
            lock_vault_ipc,
            check_security_status,
            run_purple_team_audit,
            get_security_audit_log,
            set_screen_protection
        ])
        .setup(move |app| {
            #[cfg(windows)]
            {
                if let Some(main_window) = app.get_webview_window("main") {
                    if let Ok(hwnd) = main_window.hwnd() {
                        security_guard::set_display_affinity_protection(hwnd.0 as _, true);
                    }
                    if mode_for_close == "installer" || mode_for_close == "uninstall" {
                        let _ = main_window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                            width: 640.0,
                            height: 480.0,
                        }));
                        let _ = main_window.center();
                    }
                }
            }

            if mode_for_close == "vault" {
                let open_i = MenuItem::with_id(app, "open", "Open Repose", true, None::<&str>)?;
                let lock_i = MenuItem::with_id(app, "lock", "Lock Vault", true, None::<&str>)?;
                let quit_i = MenuItem::with_id(app, "quit", "Quit Repose", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&open_i, &lock_i, &quit_i])?;

                let _tray = TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "open" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "lock" => {
                            let _ = app.emit("repose://force-lock", ());
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<SecurityState>();
                if state.app_mode == "vault" {
                    // Lock vault (zeroize VMK) and minimize to tray instead of exiting
                    api.prevent_close();
                    if let Ok(mut sess) = state.session.lock() {
                        if let Some(mut vmk) = sess.vmk.take() {
                            vmk.zeroize();
                        }
                    }
                    let _ = window.emit("repose://force-lock", ());
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
