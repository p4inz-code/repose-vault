use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use subtle::ConstantTimeEq;

const REPO_API_LATEST: &str = "https://api.github.com/repos/p4inz-code/repose-vault/releases/latest";

#[derive(Debug, Clone, Serialize)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_url: String,
    pub setup_url: Option<String>,
    pub setup_name: Option<String>,
    pub sha256sums_url: Option<String>,
}

/// Parse semver tuple (major, minor, patch) from tags like "v1.0.0" or "1.0.0"
pub fn parse_semver(tag: &str) -> Option<(u64, u64, u64)> {
    let clean = tag.trim().trim_start_matches('v');
    let parts: Vec<&str> = clean.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

pub fn is_newer_version(current: &str, latest: &str) -> bool {
    match (parse_semver(current), parse_semver(latest)) {
        (Some(c), Some(l)) => l > c,
        _ => false,
    }
}

/// Constant-time SHA-256 verification against expected hex digest
pub fn verify_sha256_bytes(data: &[u8], expected_hex: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let actual_hex = hex::encode(hasher.finalize());
    let expected_norm = expected_hex.trim().to_ascii_lowercase();
    if actual_hex.len() != expected_norm.len() {
        return false;
    }
    actual_hex
        .as_bytes()
        .ct_eq(expected_norm.as_bytes())
        .into()
}

/// Extract the expected SHA-256 hex for `filename` from a SHA256SUMS.txt file
pub fn parse_sha256sums(sums_text: &str, filename: &str) -> Option<String> {
    for line in sums_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let hash = parts.next()?;
        let name = parts.next()?;
        if name.trim_start_matches('*') == filename && hash.len() == 64 {
            return Some(hash.to_string());
        }
    }
    None
}

/// Plain HTTPS GET to GitHub Releases API (zero user data, only called on explicit user opt-in)
pub fn check_latest_release(current_version: &str) -> Result<UpdateCheckResult, String> {
    let mut cmd = std::process::Command::new("curl");
    cmd.args([
        "-fsSL",
        "--max-time",
        "10",
        "-H",
        "Accept: application/vnd.github+json",
        "-H",
        "User-Agent: Repose-Updater/1.0",
        REPO_API_LATEST,
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to invoke curl: {}", e))?;
    if !output.status.success() {
        return Err("Unable to reach GitHub Releases (offline or rate-limited).".into());
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|_| "Invalid release JSON".to_string())?;
    let tag_name = json["tag_name"]
        .as_str()
        .ok_or_else(|| "Missing tag_name in release".to_string())?;
    let release_url = json["html_url"]
        .as_str()
        .unwrap_or("https://github.com/p4inz-code/repose-vault/releases/latest")
        .to_string();

    let mut setup_url = None;
    let mut setup_name = None;
    let mut sha256sums_url = None;

    if let Some(assets) = json["assets"].as_array() {
        for asset in assets {
            let name = asset["name"].as_str().unwrap_or("");
            let dl = asset["browser_download_url"].as_str().unwrap_or("");
            if name.starts_with("Repose-Setup-") && name.ends_with(".exe") {
                setup_name = Some(name.to_string());
                setup_url = Some(dl.to_string());
            } else if name == "SHA256SUMS.txt" {
                sha256sums_url = Some(dl.to_string());
            }
        }
    }

    Ok(UpdateCheckResult {
        current_version: current_version.to_string(),
        latest_version: tag_name.to_string(),
        update_available: is_newer_version(current_version, tag_name),
        release_url,
        setup_url,
        setup_name,
        sha256sums_url,
    })
}

fn curl_download_to(url: &str, dest: &PathBuf) -> Result<(), String> {
    let mut cmd = std::process::Command::new("curl");
    cmd.args([
        "-fsSL",
        "--max-time",
        "60",
        "-o",
        &dest.display().to_string(),
        url,
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let status = cmd
        .status()
        .map_err(|e| format!("Download failed: {}", e))?;
    if !status.success() {
        return Err(format!("Download failed for {}", url));
    }
    Ok(())
}

/// Downloads Setup.exe + SHA256SUMS.txt, verifies SHA-256 before launching silent Setup
pub fn download_verify_and_launch(
    setup_url: &str,
    sha256sums_url: &str,
    setup_name: &str,
) -> Result<String, String> {
    let temp_dir = std::env::temp_dir().join("repose-update");
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let sums_path = temp_dir.join("SHA256SUMS.txt");
    let setup_path = temp_dir.join(setup_name);

    curl_download_to(sha256sums_url, &sums_path)?;
    curl_download_to(setup_url, &setup_path)?;

    let sums_text = fs::read_to_string(&sums_path).map_err(|e| e.to_string())?;
    let expected_hash = parse_sha256sums(&sums_text, setup_name)
        .ok_or_else(|| format!("Checksum for {} not found in SHA256SUMS.txt", setup_name))?;

    let setup_bytes = fs::read(&setup_path).map_err(|e| e.to_string())?;
    if !verify_sha256_bytes(&setup_bytes, &expected_hash) {
        let _ = fs::remove_file(&setup_path);
        return Err("SECURITY ALERT: Downloaded installer SHA-256 mismatch! Aborted.".into());
    }

    // Verified: launch silent custom installer (repose.exe --installer --silent)
    std::process::Command::new(&setup_path)
        .arg("/SILENT")
        .spawn()
        .map_err(|e| format!("Failed to launch verified installer: {}", e))?;

    Ok(format!("Verified SHA-256 ({}). Launching update...", &expected_hash[..12]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_comparison() {
        assert!(is_newer_version("1.0.0", "v1.0.1"));
        assert!(is_newer_version("v1.0.0", "v1.1.0"));
        assert!(!is_newer_version("1.0.0", "v1.0.0"));
        assert!(!is_newer_version("1.2.0", "v1.1.9"));
    }

    #[test]
    fn test_sha256_parse_and_constant_time_verify() {
        let payload = b"repose-verified-payload";
        let mut h = Sha256::new();
        h.update(payload);
        let hex_digest = hex::encode(h.finalize());

        let sums = format!("{}  Repose-Setup-1.0.1.exe\n", hex_digest);
        let parsed = parse_sha256sums(&sums, "Repose-Setup-1.0.1.exe").unwrap();
        assert!(verify_sha256_bytes(payload, &parsed));
        assert!(!verify_sha256_bytes(b"tampered", &parsed));
    }
}
