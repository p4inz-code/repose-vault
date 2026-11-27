use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use hkdf::Hkdf;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const ENVELOPE_MAGIC: &[u8; 8] = b"REPOSE01";

#[derive(thiserror::Error, Debug)]
pub enum CryptoError {
    #[error("Argon2 KDF failure")]
    KdfError,
    #[error("AEAD encryption or decryption failure")]
    AeadError,
    #[error("Corrupted or tampered cryptographic header")]
    HeaderTampered,
    #[error("Serialization failure")]
    SerializationError,
}

/// 32-byte memory-zeroized vault master key (VMK)
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct VaultKey([u8; 32]);

impl VaultKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self(key)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeySlotKind {
    Passphrase,
    QuickPin,
    RecoveryKey,
    Biometric,
}

/// Independent wrapped copy of the 256-bit Vault Master Key (VMK)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeySlot {
    pub kind: KeySlotKind,
    pub salt_hex: String,
    pub nonce_hex: String,
    pub wrapped_vmk_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultHeader {
    pub version: u32,
    pub kdf: String,
    pub mem_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub payload_nonce_hex: String,
    pub slots: Vec<KeySlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEnvelope {
    pub magic: String,
    pub header: VaultHeader,
    pub ciphertext_hex: String,
}

/// Derive a 256-bit wrap key using memory-hard Argon2id (64 MiB, t=3, p=4)
pub fn derive_key_argon2id(password: &str, salt: &[u8]) -> Result<VaultKey, CryptoError> {
    let params = Params::new(65536, 3, 4, Some(32)).map_err(|_| CryptoError::KdfError)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut out = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut out)
        .map_err(|_| CryptoError::KdfError)?;

    Ok(VaultKey::new(out))
}

/// Derive subkeys via HKDF-SHA256 with domain separation
pub fn derive_subkey(key: &VaultKey, domain_info: &[u8]) -> VaultKey {
    let hk = Hkdf::<Sha256>::new(None, key.as_bytes());
    let mut subkey = [0u8; 32];
    hk.expand(domain_info, &mut subkey)
        .expect("32 bytes is valid length for HKDF-SHA256");
    VaultKey::new(subkey)
}

/// Encrypt data with XChaCha20-Poly1305 using authenticated associated data (AAD)
pub fn encrypt_xchacha20(
    key: &VaultKey,
    plaintext: &[u8],
    aad: &[u8],
) -> Result<(Vec<u8>, [u8; 24]), CryptoError> {
    let cipher = XChaCha20Poly1305::new(key.as_bytes().into());

    let mut nonce_bytes = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let payload = Payload {
        msg: plaintext,
        aad,
    };

    let ciphertext = cipher
        .encrypt(nonce, payload)
        .map_err(|_| CryptoError::AeadError)?;

    Ok((ciphertext, nonce_bytes))
}

/// Decrypt data with XChaCha20-Poly1305, validating AAD authentication tag
pub fn decrypt_xchacha20(
    key: &VaultKey,
    ciphertext: &[u8],
    nonce_bytes: &[u8; 24],
    aad: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = XChaCha20Poly1305::new(key.as_bytes().into());
    let nonce = XNonce::from_slice(nonce_bytes);

    let payload = Payload {
        msg: ciphertext,
        aad,
    };

    cipher
        .decrypt(nonce, payload)
        .map_err(|_| CryptoError::AeadError)
}

/// Wrap the Vault Master Key (VMK) into an independent KeySlot for a given credential
pub fn wrap_vmk_slot(
    vmk: &VaultKey,
    kind: KeySlotKind,
    credential: &str,
) -> Result<KeySlot, CryptoError> {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);

    let kek = derive_key_argon2id(credential, &salt)?;
    let aad = format!("REPOSE_SLOT_{:?}", kind);
    let (wrapped, nonce) = encrypt_xchacha20(&kek, vmk.as_bytes(), aad.as_bytes())?;

    Ok(KeySlot {
        kind,
        salt_hex: hex::encode(salt),
        nonce_hex: hex::encode(nonce),
        wrapped_vmk_hex: hex::encode(wrapped),
    })
}

/// Unwrap the Vault Master Key (VMK) from any matching KeySlot
pub fn unwrap_vmk_slot(slot: &KeySlot, credential: &str) -> Result<VaultKey, CryptoError> {
    let salt = hex::decode(&slot.salt_hex).map_err(|_| CryptoError::HeaderTampered)?;
    let nonce_vec = hex::decode(&slot.nonce_hex).map_err(|_| CryptoError::HeaderTampered)?;
    let wrapped = hex::decode(&slot.wrapped_vmk_hex).map_err(|_| CryptoError::HeaderTampered)?;

    if nonce_vec.len() != 24 {
        return Err(CryptoError::HeaderTampered);
    }
    let mut nonce = [0u8; 24];
    nonce.copy_from_slice(&nonce_vec);

    let kek = derive_key_argon2id(credential, &salt)?;
    let aad = format!("REPOSE_SLOT_{:?}", slot.kind);
    let vmk_bytes = decrypt_xchacha20(&kek, &wrapped, &nonce, aad.as_bytes())?;
    if vmk_bytes.len() != 32 {
        return Err(CryptoError::AeadError);
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&vmk_bytes);
    Ok(VaultKey::new(out))
}

/// Seal plaintext vault JSON into a complete REPOSE01 envelope bound by AAD
pub fn seal_vault_envelope(
    vmk: &VaultKey,
    slots: Vec<KeySlot>,
    plaintext_json: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let mut nonce_bytes = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);

    let header = VaultHeader {
        version: 1,
        kdf: "Argon2id".to_string(),
        mem_kib: 65536,
        iterations: 3,
        parallelism: 4,
        payload_nonce_hex: hex::encode(nonce_bytes),
        slots,
    };

    let aad = serde_json::to_vec(&header).map_err(|_| CryptoError::SerializationError)?;
    let cipher = XChaCha20Poly1305::new(vmk.as_bytes().into());
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce_bytes),
            Payload {
                msg: plaintext_json,
                aad: &aad,
            },
        )
        .map_err(|_| CryptoError::AeadError)?;

    let envelope = VaultEnvelope {
        magic: "REPOSE01".to_string(),
        header,
        ciphertext_hex: hex::encode(ciphertext),
    };

    serde_json::to_vec(&envelope).map_err(|_| CryptoError::SerializationError)
}

/// Open and authenticate a REPOSE01 envelope using the unwrapped VMK
pub fn open_vault_envelope(
    envelope_bytes: &[u8],
    vmk: &VaultKey,
) -> Result<Vec<u8>, CryptoError> {
    let envelope: VaultEnvelope =
        serde_json::from_slice(envelope_bytes).map_err(|_| CryptoError::HeaderTampered)?;
    if envelope.magic != "REPOSE01" {
        return Err(CryptoError::HeaderTampered);
    }

    let nonce_vec =
        hex::decode(&envelope.header.payload_nonce_hex).map_err(|_| CryptoError::HeaderTampered)?;
    if nonce_vec.len() != 24 {
        return Err(CryptoError::HeaderTampered);
    }
    let mut nonce = [0u8; 24];
    nonce.copy_from_slice(&nonce_vec);

    let ciphertext =
        hex::decode(&envelope.ciphertext_hex).map_err(|_| CryptoError::HeaderTampered)?;
    let aad = serde_json::to_vec(&envelope.header).map_err(|_| CryptoError::SerializationError)?;

    decrypt_xchacha20(vmk, &ciphertext, &nonce, &aad)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2id_and_xchacha20_roundtrip() {
        let password = "SuperSecretMasterPassphrase2026!";
        let salt = [0x42u8; 32];
        let aad = b"REPOSE_VAULT_AAD_V1";
        let secret_payload = b"root:super_secret_ssh_key_ed25519";

        let key = derive_key_argon2id(password, &salt).expect("Key derivation should succeed");
        let (ciphertext, nonce) = encrypt_xchacha20(&key, secret_payload, aad).unwrap();

        let decrypted = decrypt_xchacha20(&key, &ciphertext, &nonce, aad).unwrap();
        assert_eq!(decrypted, secret_payload);

        let mut tampered = ciphertext.clone();
        tampered[0] ^= 0x01;
        assert!(decrypt_xchacha20(&key, &tampered, &nonce, aad).is_err());
        assert!(decrypt_xchacha20(&key, &ciphertext, &nonce, b"WRONG_AAD").is_err());
    }

    #[test]
    fn test_two_tier_vmk_key_slots_and_envelope() {
        let vmk = VaultKey::generate();
        let pass_slot = wrap_vmk_slot(&vmk, KeySlotKind::Passphrase, "MasterPass123!").unwrap();
        let pin_slot = wrap_vmk_slot(&vmk, KeySlotKind::QuickPin, "123456").unwrap();

        let sealed = seal_vault_envelope(&vmk, vec![pass_slot.clone(), pin_slot.clone()], b"[{\"id\":\"1\"}]").unwrap();
        let env: VaultEnvelope = serde_json::from_slice(&sealed).unwrap();

        // Both Passphrase and Quick PIN unwrap the exact same VMK and decrypt the envelope
        let unwrapped_via_pin = unwrap_vmk_slot(&env.header.slots[1], "123456").unwrap();
        let opened = open_vault_envelope(&sealed, &unwrapped_via_pin).unwrap();
        assert_eq!(opened, b"[{\"id\":\"1\"}]");

        // Wrong PIN fails cleanly
        assert!(unwrap_vmk_slot(&env.header.slots[1], "000000").is_err());
    }
}
