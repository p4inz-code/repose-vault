# Repose

**Airgapped, memory-hard desktop space vault** built with **Rust**, **Tauri v2**, **Argon2id**, and **XChaCha20-Poly1305**.

Zero telemetry. Zero cloud sync. Zero network calls (`connect-src 'none'`). Licensed under **GNU GPL v3.0**.

---

## Core Architecture

### Cryptography & Storage
- **2-Tier Key Hierarchy:** A random 256-bit **Vault Master Key (VMK)** encrypts the vault payload and is wrapped into independent slots (`Passphrase`, `QuickPin`, `RecoveryKey`, `Biometric`).
- **Memory-Hard KDF:** `Argon2id` ($m = 64\text{ MiB}$, $t = 3$, $p = 4$).
- **Authenticated Encryption:** `XChaCha20-Poly1305` with 192-bit random nonces and canonical header Associated Authenticated Data (AAD).
- **Crash-Safe Atomic I/O:** Writes to `vault.repose.tmp`, flushes with `sync_all()`, rotates 5 rolling `.bak` snapshots, and performs an atomic OS rename.
- **Encrypted Backups Only:** Plaintext JSON export is disabled; backups export as encrypted `REPOSE01` archives.

### Red / Blue / Purple Team Defenses
- **Anti-Debug:** Detects `IsDebuggerPresent()`, `CheckRemoteDebuggerPresent()`, and Linux `TracerPid`, immediately zeroizing active `VaultKey` buffers and locking the UI.
- **Anti-Paging:** Pins sensitive memory pages into physical RAM via `VirtualLock` (Windows) / `mlock()` (Unix) to prevent swapping to `pagefile.sys`.
- **Anti-Screen Capture:** Calls Win32 `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` on Windows so screen scrapers and recorders see only a black rectangle.
- **Tamper-Evident Blue Team Ledger:** SHA-256 hash-chained local audit log (`audit.ledger`) with exponential-backoff `UnlockThrottle` against brute-force attempts.
- **Purple Team Self-Audit:** Built-in 7-check adversarial verification suite (`PT-01` through `PT-07`) executable from **Settings**.

### Custom Lightweight Setup (`Repose-Setup-1.0.0.exe`)
- Thin single-file wrapper that unpacks `repose.exe` and launches Repose's own frameless custom setup UI (`--installer` / `--uninstall`).
- Installs **Per-User** (`%LOCALAPPDATA%\Programs\Repose`) with **no UAC admin prompt**.
- Uninstaller asks whether to preserve or shred your encrypted `%APPDATA%\Repose\vault.repose` data.

---

## Building from Source

### Prerequisites
- Node.js 20+ & `pnpm`
- Rust toolchain (`rustc` / `cargo` 1.80+)
- Windows: Inno Setup 6 (for `Repose-Setup-1.0.0.exe`)

### Commands
```powershell
pnpm install
pnpm run build

# Run Rust crypto, I/O, Blue Team & Purple Team tests
cd src-tauri
cargo test

# Build Repose-Setup-1.0.0.exe + SHA256SUMS.txt
cd ..
.\scripts\build-setup.ps1
```

---

## License

Copyright (C) 2026 Atharva Patil, P4inz Interactive Labs.  
Licensed under the **GNU General Public License v3.0** (`GPL-3.0-only`).
