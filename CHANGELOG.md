# Changelog

## v1.0.0 — 2026-11-27

Initial public GPL-3.0 release of **Repose**.

### Added
- **2-Tier Key Hierarchy:** Random 256-bit Vault Master Key (VMK) wrapped into `Argon2id` ($64\text{ MiB}$, $t=3$, $p=4$) + `XChaCha20-Poly1305` key slots (strictly bound to user-chosen credentials, zero default PIN backdoors).
- **Red / Blue / Purple Team Defenses:** `IsDebuggerPresent` / `CheckRemoteDebuggerPresent` / Linux `TracerPid` silent zeroize & lock, `VirtualLock` / `mlock()` RAM pinning, Win32 `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`, persistent SHA-256 hash-chained `audit.ledger`, exponential-backoff `UnlockThrottle`, and 7-check Purple Team self-audit (`10/10` Rust tests passing).
- **Custom Lightweight Setup (`Repose-Setup-1.0.0.exe`):** 5.3 MB single-file installer launching Repose's own frameless Setup & Uninstall UI (Per-User `%LOCALAPPDATA%\Programs\Repose`, zero UAC required).
- **SHA-256 Verified Auto-Updater:** Toggleable read-only GitHub release check with constant-time SHA-256 verification before silent update.
- **3-Pane Space Vault UI:** `980×640` resizable splitters, physical keyboard PIN entry, live `Ctrl+K` Command Palette, TOTP 2FA, configurable auto-clearing clipboard, encrypted `.repose` backups only, and Dark/Light themes.
