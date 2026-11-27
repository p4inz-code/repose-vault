use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityEventKind {
    VaultInitialized,
    VaultUnlocked,
    UnlockFailed,
    UnlockThrottled,
    VaultLocked,
    DebuggerDetected,
    InjectionEnvDetected,
    HeaderTamperDetected,
    BackupExported,
    ClipboardCleared,
    PurpleTeamAuditRun,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub seq: u64,
    pub timestamp_unix: u64,
    pub kind: SecurityEventKind,
    pub detail: String,
    pub prev_hash: String,
    pub entry_hash: String,
}

/// Tamper-evident SHA-256 hash-chained local security ledger (zero telemetry, 100% offline)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityAuditLog {
    pub entries: Vec<AuditEntry>,
}

impl SecurityAuditLog {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    fn compute_hash(
        seq: u64,
        timestamp_unix: u64,
        kind: &SecurityEventKind,
        detail: &str,
        prev_hash: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(seq.to_le_bytes());
        hasher.update(timestamp_unix.to_le_bytes());
        hasher.update(format!("{:?}", kind).as_bytes());
        hasher.update(detail.as_bytes());
        hasher.update(prev_hash.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn record(&mut self, kind: SecurityEventKind, detail: impl Into<String>) -> AuditEntry {
        let seq = self.entries.len() as u64 + 1;
        let timestamp_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let prev_hash = self
            .entries
            .last()
            .map(|e| e.entry_hash.clone())
            .unwrap_or_else(|| "0".repeat(64));
        let detail_str = detail.into();
        let entry_hash = Self::compute_hash(seq, timestamp_unix, &kind, &detail_str, &prev_hash);

        let entry = AuditEntry {
            seq,
            timestamp_unix,
            kind,
            detail: detail_str,
            prev_hash,
            entry_hash,
        };
        self.entries.push(entry.clone());
        entry
    }

    /// Recomputes every hash in the ledger to detect deletion, reordering, or modification
    pub fn verify_chain(&self) -> Result<(), String> {
        let mut expected_prev = "0".repeat(64);
        for (idx, entry) in self.entries.iter().enumerate() {
            let expected_seq = idx as u64 + 1;
            if entry.seq != expected_seq {
                return Err(format!("Sequence gap at index {}: expected {}", idx, expected_seq));
            }
            if entry.prev_hash != expected_prev {
                return Err(format!("Broken prev_hash link at seq {}", entry.seq));
            }
            let recomputed = Self::compute_hash(
                entry.seq,
                entry.timestamp_unix,
                &entry.kind,
                &entry.detail,
                &entry.prev_hash,
            );
            if entry.entry_hash != recomputed {
                return Err(format!("Tampered entry_hash at seq {}", entry.seq));
            }
            expected_prev = entry.entry_hash.clone();
        }
        Ok(())
    }

    /// Persist hash-chained audit ledger to disk
    pub fn save_to_disk(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let tmp = path.with_extension("ledger.tmp");
        std::fs::write(&tmp, &json)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Load hash-chained audit ledger from disk and verify integrity
    pub fn load_from_disk(path: &std::path::Path) -> Self {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(parsed) = serde_json::from_slice::<Self>(&bytes) {
                if parsed.verify_chain().is_ok() {
                    return parsed;
                }
            }
        }
        Self::new()
    }
}

/// Exponential backoff rate limiter against brute-force / automated credential stuffing
#[derive(Debug, Clone)]
pub struct UnlockThrottle {
    pub failed_attempts: u32,
    pub locked_until_unix: u64,
}

impl Default for UnlockThrottle {
    fn default() -> Self {
        Self {
            failed_attempts: 0,
            locked_until_unix: 0,
        }
    }
}

impl UnlockThrottle {
    pub fn check_allowed(&self, now_unix: u64) -> Result<(), u64> {
        if now_unix < self.locked_until_unix {
            Err(self.locked_until_unix - now_unix)
        } else {
            Ok(())
        }
    }

    pub fn record_failure(&mut self, now_unix: u64) -> u64 {
        self.failed_attempts = self.failed_attempts.saturating_add(1);
        let penalty_secs = match self.failed_attempts {
            1..=2 => 0,
            3 => 2,
            4 => 5,
            5 => 15,
            6 => 30,
            _ => 60,
        };
        self.locked_until_unix = now_unix.saturating_add(penalty_secs);
        penalty_secs
    }

    pub fn record_success(&mut self) {
        self.failed_attempts = 0;
        self.locked_until_unix = 0;
    }
}

/// Detect dangerous dynamic-linker / profiler injection environment variables
pub fn detect_injection_environment() -> Vec<String> {
    const SUSPICIOUS_VARS: &[&str] = &[
        "LD_PRELOAD",
        "LD_AUDIT",
        "DYLD_INSERT_LIBRARIES",
        "DYLD_FORCE_FLAT_NAMESPACE",
        "COR_ENABLE_PROFILING",
        "CORECLR_ENABLE_PROFILING",
        "DOTNET_STARTUP_HOOKS",
    ];

    SUSPICIOUS_VARS
        .iter()
        .filter_map(|var| {
            std::env::var(var)
                .ok()
                .filter(|v| !v.is_empty())
                .map(|_| (*var).to_string())
        })
        .collect()
}

/// Compute SHA-256 fingerprint of clipboard content for safe ownership-verified auto-wipe
pub fn clipboard_fingerprint(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_chain_detects_tampering() {
        let mut log = SecurityAuditLog::new();
        log.record(SecurityEventKind::VaultInitialized, "Created vault");
        log.record(SecurityEventKind::VaultUnlocked, "Auth via passphrase");
        log.record(SecurityEventKind::VaultLocked, "Idle timeout");

        assert!(log.verify_chain().is_ok());

        // Simulate attacker editing entry 2
        log.entries[1].detail = "Auth via bypass".to_string();
        let err = log.verify_chain().unwrap_err();
        assert!(err.contains("Tampered entry_hash at seq 2"));
    }

    #[test]
    fn test_unlock_throttle_exponential_backoff() {
        let mut throttle = UnlockThrottle::default();
        assert_eq!(throttle.record_failure(100), 0);
        assert_eq!(throttle.record_failure(100), 0);
        assert_eq!(throttle.record_failure(100), 2);
        assert_eq!(throttle.check_allowed(101), Err(1));
        assert_eq!(throttle.check_allowed(102), Ok(()));

        throttle.record_success();
        assert_eq!(throttle.failed_attempts, 0);
    }
}
