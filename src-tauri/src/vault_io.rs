use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const MAX_ROLLING_BACKUPS: usize = 5;

/// Resolve native platform-specific data storage directory for Repose:
/// - Windows: %APPDATA%\Repose\
/// - macOS: ~/Library/Application Support/Repose/
/// - Linux: ~/.local/share/repose/
pub fn get_vault_storage_dir() -> io::Result<PathBuf> {
    let base_dir = dirs::data_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Unable to resolve OS user data directory")
    })?;

    let app_dir = base_dir.join("Repose");
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }
    Ok(app_dir)
}

/// Default primary vault file path
pub fn get_default_vault_path() -> io::Result<PathBuf> {
    let dir = get_vault_storage_dir()?;
    Ok(dir.join("vault.repose"))
}

/// Rotate rolling backups: vault.repose.bak.4 -> .5, etc.
fn rotate_backups(vault_path: &Path) -> io::Result<()> {
    if !vault_path.exists() {
        return Ok(());
    }

    // Shift older backups down
    for i in (1..MAX_ROLLING_BACKUPS).rev() {
        let from = vault_path.with_extension(format!("repose.bak.{}", i));
        let to = vault_path.with_extension(format!("repose.bak.{}", i + 1));
        if from.exists() {
            let _ = fs::rename(&from, &to);
        }
    }

    // Current file becomes bak.1
    let bak1 = vault_path.with_extension("repose.bak.1");
    let _ = fs::copy(vault_path, &bak1);
    Ok(())
}

/// Crash-safe atomic save:
/// 1. Write to temporary file `vault.repose.tmp`
/// 2. Flush and sync to disk (`sync_all()`)
/// 3. Rotate rolling backup copies
/// 4. Atomic rename temporary file to target path
pub fn save_vault_atomic(vault_path: &Path, data: &[u8]) -> io::Result<()> {
    let parent = vault_path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Invalid vault path parent")
    })?;
    fs::create_dir_all(parent)?;

    let tmp_path = vault_path.with_extension("repose.tmp");

    {
        let mut file = File::create(&tmp_path)?;
        file.write_all(data)?;
        file.sync_all()?; // Force kernel write to physical block device
    }

    // Safety: maintain rolling snapshots before overwriting target
    let _ = rotate_backups(vault_path);

    // Atomic OS replace
    fs::rename(&tmp_path, vault_path)?;

    Ok(())
}

/// Read vault ciphertext bytes from disk
pub fn read_vault_file(vault_path: &Path) -> io::Result<Vec<u8>> {
    fs::read(vault_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_save_and_read() {
        let temp_dir = std::env::temp_dir().join("repose_test_io");
        let vault_file = temp_dir.join("test_vault.repose");

        let test_payload = b"REPOSE01_ENCRYPTED_PAYLOAD_TEST";
        save_vault_atomic(&vault_file, test_payload).expect("Atomic save failed");

        let read_back = read_vault_file(&vault_file).expect("Read back failed");
        assert_eq!(read_back, test_payload);

        // Save again to trigger backup rotation
        let test_payload_v2 = b"REPOSE01_ENCRYPTED_PAYLOAD_V2";
        save_vault_atomic(&vault_file, test_payload_v2).expect("Atomic save v2 failed");

        let bak1 = vault_file.with_extension("repose.bak.1");
        assert!(bak1.exists(), "Backup 1 should exist");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
