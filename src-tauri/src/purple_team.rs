use serde::Serialize;

use crate::blue_team::{
    detect_injection_environment, SecurityAuditLog, SecurityEventKind, UnlockThrottle,
};
use crate::crypto::{
    decrypt_xchacha20, derive_key_argon2id, encrypt_xchacha20, VaultKey,
};
use crate::security_guard::{
    is_debugger_attached, lock_memory_buffer, unlock_memory_buffer,
};

#[derive(Debug, Clone, Serialize)]
pub struct PurpleCheckResult {
    pub id: String,
    pub name: String,
    pub attack_vector: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PurpleTeamReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub checks: Vec<PurpleCheckResult>,
}

/// Executes live Red/Blue adversarial self-tests to prove security invariants hold
pub fn run_purple_team_suite() -> PurpleTeamReport {
    let mut checks = Vec::new();

    // 1. AEAD Header Tamper Attack
    let header_tamper = (|| -> Result<String, String> {
        let key = VaultKey::generate();
        let aad = b"REPOSE01|v1|header";
        let (ct, nonce) = encrypt_xchacha20(&key, b"secret-payload", aad)
            .map_err(|e| e.to_string())?;
        let tampered_aad = b"REPOSE01|v1|headex";
        match decrypt_xchacha20(&key, &ct, &nonce, tampered_aad) {
            Err(_) => Ok("XChaCha20-Poly1305 rejected 1-bit AAD header mutation".into()),
            Ok(_) => Err("Decryption succeeded with tampered AAD!".into()),
        }
    })();
    checks.push(PurpleCheckResult {
        id: "PT-01".into(),
        name: "AEAD Header Tamper Rejection".into(),
        attack_vector: "Bit-flip in unencrypted vault header metadata".into(),
        passed: header_tamper.is_ok(),
        detail: header_tamper.unwrap_or_else(|e| e),
    });

    // 2. Ciphertext Bit-Flip Attack
    let ct_tamper = (|| -> Result<String, String> {
        let key = VaultKey::generate();
        let aad = b"REPOSE01|v1|header";
        let (mut ct, nonce) = encrypt_xchacha20(&key, b"secret-payload", aad)
            .map_err(|e| e.to_string())?;
        ct[0] ^= 0xFF;
        match decrypt_xchacha20(&key, &ct, &nonce, aad) {
            Err(_) => Ok("Poly1305 MAC rejected mutated ciphertext byte".into()),
            Ok(_) => Err("Mutated ciphertext decrypted!".into()),
        }
    })();
    checks.push(PurpleCheckResult {
        id: "PT-02".into(),
        name: "Ciphertext Integrity Check".into(),
        attack_vector: "Direct byte corruption in encrypted vault payload".into(),
        passed: ct_tamper.is_ok(),
        detail: ct_tamper.unwrap_or_else(|e| e),
    });

    // 3. Wrong-Key Decryption Rejection (Argon2id domain separation)
    let wrong_key = (|| -> Result<String, String> {
        let salt = b"repose-purple-team-salt-16b";
        let k1 = derive_key_argon2id("CorrectPassphrase!", salt).map_err(|e| e.to_string())?;
        let k2 = derive_key_argon2id("WrongPassphrase!", salt).map_err(|e| e.to_string())?;
        let aad = b"REPOSE01";
        let (ct, nonce) = encrypt_xchacha20(&k1, b"vault-data", aad).map_err(|e| e.to_string())?;
        match decrypt_xchacha20(&k2, &ct, &nonce, aad) {
            Err(_) => Ok("Argon2id + XChaCha20 rejected wrong passphrase cleanly".into()),
            Ok(_) => Err("Wrong passphrase decrypted vault!".into()),
        }
    })();
    checks.push(PurpleCheckResult {
        id: "PT-03".into(),
        name: "Wrong-Key KDF Rejection".into(),
        attack_vector: "Credential guessing against Argon2id (64 MiB)".into(),
        passed: wrong_key.is_ok(),
        detail: wrong_key.unwrap_or_else(|e| e),
    });

    // 4. Physical RAM Page Locking (Anti-Swap / Anti-Pagefile)
    let page_lock = {
        let buf = vec![0xA5u8; 4096];
        let locked = lock_memory_buffer(&buf);
        let unlocked = unlock_memory_buffer(&buf);
        (locked && unlocked, locked, unlocked)
    };
    checks.push(PurpleCheckResult {
        id: "PT-04".into(),
        name: "Physical RAM Page Lock (VirtualLock/mlock)".into(),
        attack_vector: "OS paging decrypted keys to pagefile.sys / swap".into(),
        passed: page_lock.0,
        detail: format!("lock={}, unlock={}", page_lock.1, page_lock.2),
    });

    // 5. Debugger & Injection Environment Probe
    let debugger = is_debugger_attached();
    let inj_vars = detect_injection_environment();
    let clean_env = !debugger && inj_vars.is_empty();
    checks.push(PurpleCheckResult {
        id: "PT-05".into(),
        name: "Runtime Anti-Debug & DLL Injection Probe".into(),
        attack_vector: "x64dbg / WinDbg attachment or LD_PRELOAD/DYLD injection".into(),
        passed: clean_env,
        detail: if clean_env {
            "No active debugger or injection env vars detected".into()
        } else {
            format!("debugger={}, injection_vars={:?}", debugger, inj_vars)
        },
    });

    // 6. Blue Team Audit Ledger Tamper Detection
    let ledger_check = (|| -> Result<String, String> {
        let mut log = SecurityAuditLog::new();
        log.record(SecurityEventKind::VaultInitialized, "init");
        log.record(SecurityEventKind::VaultUnlocked, "unlock");
        log.entries[0].detail = "forged".into();
        match log.verify_chain() {
            Err(reason) => Ok(format!("Chain verified tampering: {}", reason)),
            Ok(_) => Err("Audit log failed to detect forged entry!".into()),
        }
    })();
    checks.push(PurpleCheckResult {
        id: "PT-06".into(),
        name: "Audit Ledger Tamper Detection".into(),
        attack_vector: "Attacker alters local security audit history".into(),
        passed: ledger_check.is_ok(),
        detail: ledger_check.unwrap_or_else(|e| e),
    });

    // 7. Brute-Force Exponential Backoff Enforcement
    let throttle_check = {
        let mut t = UnlockThrottle::default();
        t.record_failure(1000);
        t.record_failure(1000);
        let penalty = t.record_failure(1000);
        let blocked = t.check_allowed(1001).is_err();
        penalty >= 2 && blocked
    };
    checks.push(PurpleCheckResult {
        id: "PT-07".into(),
        name: "Brute-Force Exponential Backoff".into(),
        attack_vector: "Rapid automated PIN/passphrase guessing".into(),
        passed: throttle_check,
        detail: if throttle_check {
            "3rd consecutive failure enforced 2s+ lockout window".into()
        } else {
            "Throttle failed to enforce lockout".into()
        },
    });

    let passed = checks.iter().filter(|c| c.passed).count();
    let total = checks.len();
    PurpleTeamReport {
        total,
        passed,
        failed: total - passed,
        checks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purple_team_suite_all_pass() {
        let report = run_purple_team_suite();
        assert_eq!(report.failed, 0, "Purple team checks failed: {:?}", report.checks);
        assert_eq!(report.passed, report.total);
    }
}
