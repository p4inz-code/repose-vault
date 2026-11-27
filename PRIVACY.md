# Privacy Policy

Repose is **100% offline and airgapped**.

- **Zero Telemetry or Tracking:** Tauri's Content Security Policy enforces `connect-src 'none'` in the WebView. No telemetry, crash reports, or analytics ever leave your computer.
- **Automatic Update Check (`github.com`):** A plain read-only request for the latest release with zero user data, which you can turn off in **Settings → Automatic Release Updates**. Downloaded updates are SHA-256 verified against `SHA256SUMS.txt` before silent installation.
- **Local Storage Only:** Your encrypted vault (`vault.repose`), rolling `.bak` snapshots, and tamper-evident `audit.ledger` are stored exclusively in your OS user data directory (`%APPDATA%\Repose` on Windows).
- **Encrypted Backups Only:** Plaintext JSON export is disabled by design. All exported backups are encrypted `REPOSE01` archives.
- **Zero Cloud Accounts:** No account registration, no email, no tracking identifiers.
