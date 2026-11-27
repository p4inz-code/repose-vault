# Contributing

Contributions are welcome — Repose is free software and stays that way.

## Before you start
- Small fixes: just open a pull request.
- Bigger changes: open an issue first so we agree on the direction.
- By contributing you agree that your work is released under the project's license (GPLv3 + [additional terms](ADDITIONAL_TERMS.md)). Your authorship is kept in the git history.

## Build
- **Frontend:** `pnpm install && pnpm run build` (strictly airgapped, zero external fonts or network calls).
- **Rust Core:** `cd src-tauri && cargo test`
- **Windows Setup Installer:** `powershell -ExecutionPolicy Bypass -File .\scripts\build-setup.ps1`

## Code & Security Style
- Rust 2021 (`src-tauri/src/`) + TypeScript/React (`src/`).
- Keep the UI fully custom (no OS native dialogs or scrollbars) and keep `connect-src 'none'` strictly enforced.
- All cryptographic keys must implement `ZeroizeOnDrop` and never be written to disk in plaintext.
