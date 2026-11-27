# Changelog

## v1.0.0 — 2026-11-27

Initial public GPL-3.0 release of **Repose**.

### Added
- **2-Tier Key Hierarchy:** Random 256-bit Vault Master Key (VMK) wrapped into `Argon2id` ($64\text{ MiB}$, $t=3$, $p=4$) + `XChaCha20-Poly1305` key slots.
- **Red / Blue / Purple Team Defenses:** `IsDebuggerPresent` / `CheckRemoteDebuggerPresent` / Linux `TracerPid` silent zeroize & lock, `VirtualLock` / `mlock()` RAM pinning, Win32 `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`, persistent SHA-256 hash-chained `audit.ledger`, exponential-backoff `UnlockThrottle`, and 7-check Purple Team self-audit.
- **Custom Lightweight Setup (`Repose-Setup-1.0.0.exe`):** 5.3 MB single-file installer launching Repose's own frameless Setup & Uninstall UI (Per-User `%LOCALAPPDATA%\Programs\Repose`, zero UAC required).
- **3-Pane Space Vault UI:** Resizable splitters, physical keyboard PIN entry, live `Ctrl+K` Command Palette, TOTP 2FA, configurable auto-clearing clipboard, encrypted `.repose` backups only, and Dark/Light themes.
