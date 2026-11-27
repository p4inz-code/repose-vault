# Security Policy

## Threat Model & Invariants
Repose is an **airgapped local-first secret vault**. By design:
1. **Zero Network Calls:** `connect-src 'none'` is enforced in Tauri CSP; fonts and assets are bundled locally.
2. **Memory Zeroization:** Master keys (`VaultKey`) implement `ZeroizeOnDrop` and are scrubbed on lock, close-to-tray, or debugger detection.
3. **No Plaintext Exports:** Backups are strictly encrypted `REPOSE01` archives.

## Reporting a Vulnerability
If you discover a cryptographic, memory-handling, or installer security issue, please open a private security advisory on GitHub (`p4inz-code/repose-vault`).
