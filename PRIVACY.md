# Privacy Policy

Repose is **100% offline and airgapped**.

- **Zero Network Calls:** Tauri's Content Security Policy strictly enforces `connect-src 'none'`. No telemetry, crash reports, analytics, or update pings ever leave your computer.
- **Local Storage Only:** Your encrypted vault (`vault.repose`), rolling `.bak` snapshots, and tamper-evident `audit.ledger` are stored exclusively in your OS user data directory (`%APPDATA%\Repose` on Windows).
- **Encrypted Backups Only:** Plaintext JSON export is disabled by design. All exported backups are encrypted `REPOSE01` archives.
- **Zero Cloud Accounts:** No account registration, no email, no tracking identifiers.
