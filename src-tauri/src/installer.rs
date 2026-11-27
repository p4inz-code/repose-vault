use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::security_guard::{is_debugger_attached, lock_memory_buffer, unlock_memory_buffer};
use crate::vault_io::get_vault_storage_dir;

#[derive(Debug, Clone, Serialize)]
pub struct DoctorCheck {
    pub id: String,
    pub label: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallResult {
    pub success: bool,
    pub install_dir: String,
    pub exe_path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UninstallResult {
    pub success: bool,
    pub message: String,
}

pub fn default_install_dir() -> PathBuf {
    if let Some(local_app_data) = dirs::data_local_dir() {
        return local_app_data.join("Programs").join("Repose");
    }
    PathBuf::from("C:\\Users\\Public\\Repose")
}

pub fn is_installed_at(dir: &Path) -> bool {
    dir.join("repose.exe").exists()
}

/// Pre-flight system & security readiness checks shown in the Setup Doctor step
pub fn run_doctor_checks(target_dir: &str) -> Vec<DoctorCheck> {
    let mut checks = Vec::new();

    // 1. OS & Architecture
    checks.push(DoctorCheck {
        id: "DOC-01".into(),
        label: "Platform & Architecture".into(),
        ok: cfg!(target_arch = "x86_64") || cfg!(target_arch = "aarch64"),
        detail: format!("{} ({})", std::env::consts::OS, std::env::consts::ARCH),
    });

    // 2. Target Install Path Writable (Per-User, no UAC required)
    let dest = PathBuf::from(target_dir);
    let writable = fs::create_dir_all(&dest).is_ok();
    checks.push(DoctorCheck {
        id: "DOC-02".into(),
        label: "Per-User Install Directory".into(),
        ok: writable,
        detail: if writable {
            format!("Writable without elevation: {}", dest.display())
        } else {
            format!("Cannot create directory: {}", dest.display())
        },
    });

    // 3. Vault Data Directory
    let data_dir = get_vault_storage_dir();
    checks.push(DoctorCheck {
        id: "DOC-03".into(),
        label: "Encrypted Vault Storage".into(),
        ok: data_dir.is_ok(),
        detail: data_dir
            .map(|p| p.display().to_string())
            .unwrap_or_else(|e| e.to_string()),
    });

    // 4. Physical RAM Locking (VirtualLock / mlock)
    let probe = vec![0xAAu8; 4096];
    let locked = lock_memory_buffer(&probe);
    let _ = unlock_memory_buffer(&probe);
    checks.push(DoctorCheck {
        id: "DOC-04".into(),
        label: "RAM Page Pinning (VirtualLock)".into(),
        ok: locked,
        detail: if locked {
            "Keys can be pinned in physical RAM (excluded from pagefile)".into()
        } else {
            "OS denied physical page lock".into()
        },
    });

    // 5. Clean-Room Anti-Debug Check
    let dbg = is_debugger_attached();
    checks.push(DoctorCheck {
        id: "DOC-05".into(),
        label: "Clean-Room Environment".into(),
        ok: !dbg,
        detail: if !dbg {
            "No debugger or tracer attached".into()
        } else {
            "Debugger detected during setup".into()
        },
    });

    checks
}

/// Copies the current binary & payload into `dest`, creates shortcuts, and registers Windows Uninstall
pub fn perform_install(
    dest_dir: &str,
    desktop_shortcut: bool,
    start_menu_shortcut: bool,
) -> Result<InstallResult, String> {
    let dest = PathBuf::from(dest_dir);
    fs::create_dir_all(&dest).map_err(|e| format!("Failed to create install folder: {}", e))?;

    let current_exe =
        std::env::current_exe().map_err(|e| format!("Cannot locate setup executable: {}", e))?;
    let source_dir = current_exe
        .parent()
        .ok_or_else(|| "Invalid executable parent".to_string())?;

    let target_exe = dest.join("repose.exe");

    // Copy main executable and any sibling runtime files (excluding temp setup markers)
    fs::copy(&current_exe, &target_exe)
        .map_err(|e| format!("Failed to copy repose.exe: {}", e))?;

    if let Ok(entries) = fs::read_dir(source_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !name.eq_ignore_ascii_case("repose.exe")
                        && !name.ends_with(".tmp")
                        && !name.ends_with(".zip")
                    {
                        let _ = fs::copy(&path, dest.join(name));
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    {
        create_windows_shortcuts(&target_exe, desktop_shortcut, start_menu_shortcut)?;
        register_windows_uninstall(&dest, &target_exe)?;
    }

    #[cfg(not(windows))]
    {
        let _ = desktop_shortcut;
        let _ = start_menu_shortcut;
    }

    Ok(InstallResult {
        success: true,
        install_dir: dest.display().to_string(),
        exe_path: target_exe.display().to_string(),
        message: "Repose installed cleanly (Per-User, zero UAC).".to_string(),
    })
}

/// Removes shortcuts, Windows Uninstall registry entry, and optionally shreds vault data
pub fn perform_uninstall(shred_vault_data: bool) -> Result<UninstallResult, String> {
    #[cfg(windows)]
    {
        remove_windows_shortcuts();
        remove_windows_uninstall_reg();
    }

    if shred_vault_data {
        if let Ok(vault_dir) = get_vault_storage_dir() {
            let _ = fs::remove_dir_all(vault_dir);
        }
    }

    // Schedule self-cleanup of install directory after process exits
    let install_dir = default_install_dir();
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let cmd = format!(
            "ping -n 2 127.0.0.1 >nul & rmdir /s /q \"{}\"",
            install_dir.display()
        );
        let _ = std::process::Command::new("cmd")
            .args(["/C", &cmd])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }

    Ok(UninstallResult {
        success: true,
        message: if shred_vault_data {
            "Repose uninstalled and local encrypted vault data shredded.".to_string()
        } else {
            "Repose uninstalled. Encrypted vault data preserved in %APPDATA%\\Repose.".to_string()
        },
    })
}

#[cfg(windows)]
fn create_shortcut_via_ps(lnk_path: &Path, target_exe: &Path, desc: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let ps = format!(
        "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{}');$s.TargetPath='{}';$s.WorkingDirectory='{}';$s.Description='{}';$s.Save()",
        lnk_path.display().to_string().replace('\'', "''"),
        target_exe.display().to_string().replace('\'', "''"),
        target_exe.parent().unwrap_or(target_exe).display().to_string().replace('\'', "''"),
        desc.replace('\'', "''")
    );
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| format!("Shortcut creation failed: {}", e))?;

    if !status.success() {
        return Err("PowerShell shortcut script returned non-zero exit code".into());
    }
    Ok(())
}

#[cfg(windows)]
fn create_windows_shortcuts(
    target_exe: &Path,
    desktop_shortcut: bool,
    start_menu_shortcut: bool,
) -> Result<(), String> {
    if desktop_shortcut {
        if let Some(desktop) = dirs::desktop_dir() {
            let _ = create_shortcut_via_ps(
                &desktop.join("Repose.lnk"),
                target_exe,
                "Repose — Airgapped Space Vault",
            );
        }
    }
    if start_menu_shortcut {
        if let Some(roaming) = dirs::data_dir() {
            let start_dir = roaming
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs");
            let _ = fs::create_dir_all(&start_dir);
            let _ = create_shortcut_via_ps(
                &start_dir.join("Repose.lnk"),
                target_exe,
                "Repose — Airgapped Space Vault",
            );
        }
    }
    Ok(())
}

#[cfg(windows)]
fn remove_windows_shortcuts() {
    if let Some(desktop) = dirs::desktop_dir() {
        let _ = fs::remove_file(desktop.join("Repose.lnk"));
    }
    if let Some(roaming) = dirs::data_dir() {
        let start_lnk = roaming
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Repose.lnk");
        let _ = fs::remove_file(start_lnk);
    }
}

#[cfg(windows)]
fn register_windows_uninstall(install_dir: &Path, target_exe: &Path) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Repose";
    let (key, _) = hkcu
        .create_subkey(path)
        .map_err(|e| format!("Failed to register uninstaller: {}", e))?;

    key.set_value("DisplayName", &"Repose — Airgapped Space Vault")
        .map_err(|e| e.to_string())?;
    key.set_value("DisplayVersion", &"1.0.0")
        .map_err(|e| e.to_string())?;
    key.set_value("Publisher", &"P4inz Interactive Labs")
        .map_err(|e| e.to_string())?;
    key.set_value("InstallLocation", &install_dir.display().to_string())
        .map_err(|e| e.to_string())?;
    key.set_value("DisplayIcon", &target_exe.display().to_string())
        .map_err(|e| e.to_string())?;
    key.set_value(
        "UninstallString",
        &format!("\"{}\" --uninstall", target_exe.display()),
    )
    .map_err(|e| e.to_string())?;
    key.set_value("NoModify", &1u32).map_err(|e| e.to_string())?;
    key.set_value("NoRepair", &0u32).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(windows)]
fn remove_windows_uninstall_reg() {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let _ = hkcu.delete_subkey_all("Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Repose");
}
