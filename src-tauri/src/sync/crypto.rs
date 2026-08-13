use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::Engine;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;

/// Errors that can occur during cryptographic operations.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionError(String),
    #[error("Decryption failed: {0}")]
    DecryptionError(String),
    #[error("Invalid ciphertext format")]
    InvalidFormat,
    #[error("Unsupported envelope version")]
    UnsupportedVersion,
    #[error("Corrupted sync data")]
    CorruptedData,
}

/// PBKDF2 iterations — must match OpenTubeX server.
const PBKDF2_ITERATIONS: u32 = 600_000;

/// AES-GCM AAD — must match OpenTubeX server.
const AAD: &[u8] = b"OpenTubeX encrypted sync v1";

/// Padding block size (64 KiB) — reduces size leakage.
const PADDING_BLOCK_BYTES: usize = 64 * 1024;

/// Derives a 256-bit key from a passphrase using PBKDF2-SHA256.
///
/// Uses 600,000 iterations and a 16-byte salt to match the OpenTubeX server.
pub fn derive_key(passphrase: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

/// Imports an existing raw key (base64-encoded) for decryption.
pub fn import_key(raw: &[u8]) -> Result<[u8; 32], CryptoError> {
    if raw.len() != 32 {
        return Err(CryptoError::InvalidFormat);
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(raw);
    Ok(key)
}

/// Generates a 16-byte random salt for new accounts.
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Encrypts data using the OpenTubeX envelope format.
///
/// Process:
/// 1. Encode plaintext as JSON: `{"version": 1, "data": <input>}`
/// 2. Gzip-compress, prefixed with 4-byte big-endian length
/// 3. Pad to multiple of 64 KiB with 0x20 bytes
/// 4. AES-256-GCM encrypt with random 12-byte IV and AAD
/// 5. Serialize as JSON envelope with base64-encoded binary fields
///
/// The `salt` parameter is the account's KDF salt, stored in the envelope for
/// reference. The actual key must be derived separately using this salt.
pub fn encrypt_envelope(
    data: &serde_json::Value,
    key: &[u8; 32],
    salt: &[u8],
) -> Result<String, CryptoError> {
    // Step 1: Encode as JSON
    let plaintext = serde_json::json!({ "version": 1, "data": data });
    let plaintext_bytes = serde_json::to_vec(&plaintext)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;

    // Step 2: Gzip compress with 4-byte length prefix
    let compressed = gzip_compress(&plaintext_bytes)?;

    // Step 3: Pad to multiple of 64 KiB
    let padded = pad(&compressed);

    // Step 4: Encrypt
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let ciphertext = cipher
        .encrypt(nonce, aes_gcm::aead::Payload {
            msg: &padded,
            aad: AAD,
        })
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;

    // Step 5: Build envelope
    let envelope = serde_json::json!({
        "version": 1,
        "kdf": {
            "name": "PBKDF2",
            "hash": "SHA-256",
            "iterations": PBKDF2_ITERATIONS,
            "salt": base64::engine::general_purpose::STANDARD.encode(salt),
        },
        "cipher": {
            "name": "AES-GCM",
            "iv": base64::engine::general_purpose::STANDARD.encode(nonce_bytes),
        },
        "compression": {
            "name": "gzip",
        },
        "ciphertext": base64::engine::general_purpose::STANDARD.encode(&ciphertext),
    });

    serde_json::to_string(&envelope)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))
}

/// Extracts the salt from an encrypted envelope.
///
/// This is used when the account's salt is stored only in the envelope
/// (e.g., legacy accounts or when deriving the key for the first time).
pub fn get_salt_from_envelope(envelope_str: &str) -> Result<Vec<u8>, CryptoError> {
    let envelope: serde_json::Value = serde_json::from_str(envelope_str)
        .map_err(|_| CryptoError::CorruptedData)?;

    let salt_b64 = envelope
        .get("kdf")
        .and_then(|k| k.get("salt"))
        .and_then(|s| s.as_str())
        .ok_or(CryptoError::CorruptedData)?;

    base64::engine::general_purpose::STANDARD.decode(salt_b64).map_err(|_| CryptoError::CorruptedData)
}

/// Extracts the PBKDF2 iterations from an envelope (for validation/migration).
pub fn get_iterations_from_envelope(envelope_str: &str) -> Result<u32, CryptoError> {
    let envelope: serde_json::Value = serde_json::from_str(envelope_str)
        .map_err(|_| CryptoError::CorruptedData)?;

    envelope
        .get("kdf")
        .and_then(|k| k.get("iterations"))
        .and_then(|i| i.as_u64())
        .map(|i| i as u32)
        .ok_or(CryptoError::CorruptedData)
}
///
/// Reverse of `encrypt_envelope`:
/// 1. Parse JSON envelope
/// 2. AES-256-GCM decrypt
/// 3. Strip padding
/// 4. Gzip decompress (read 4-byte length, decompress slice)
/// 5. Parse JSON, return `data` field
pub fn decrypt_envelope(envelope_str: &str, key: &[u8; 32]) -> Result<serde_json::Value, CryptoError> {
    // Step 1: Parse envelope
    let envelope: serde_json::Value = serde_json::from_str(envelope_str)
        .map_err(|_| CryptoError::CorruptedData)?;

    let version = envelope.get("version")
        .and_then(|v| v.as_u64())
        .ok_or(CryptoError::CorruptedData)?;

    if version != 1 {
        return Err(CryptoError::UnsupportedVersion);
    }

    // Validate KDF
    let kdf = envelope.get("kdf").ok_or(CryptoError::CorruptedData)?;
    if kdf.get("name").and_then(|n| n.as_str()) != Some("PBKDF2")
        || kdf.get("hash").and_then(|h| h.as_str()) != Some("SHA-256")
        || kdf.get("iterations").and_then(|i| i.as_u64()) != Some(PBKDF2_ITERATIONS as u64)
    {
        return Err(CryptoError::UnsupportedVersion);
    }

    // Validate cipher
    let cipher = envelope.get("cipher").ok_or(CryptoError::CorruptedData)?;
    if cipher.get("name").and_then(|n| n.as_str()) != Some("AES-GCM") {
        return Err(CryptoError::UnsupportedVersion);
    }

    let iv = cipher.get("iv")
        .and_then(|v| v.as_str())
        .and_then(|s| base64::engine::general_purpose::STANDARD.decode(s).ok())
        .and_then(|v| {
            if v.len() == 12 { Some(v) } else { None }
        })
        .ok_or(CryptoError::CorruptedData)?;

    let ciphertext_b64 = envelope.get("ciphertext")
        .and_then(|v| v.as_str())
        .ok_or(CryptoError::CorruptedData)?;
    let ciphertext = base64::engine::general_purpose::STANDARD.decode(ciphertext_b64)
        .map_err(|_| CryptoError::CorruptedData)?;

    // Step 2: Decrypt
    let nonce = Nonce::from_slice(&iv);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let padded = cipher
        .decrypt(nonce, aes_gcm::aead::Payload {
            msg: &ciphertext,
            aad: AAD,
        })
        .map_err(|_| CryptoError::CorruptedData)?;

    // Step 3: Strip padding (find last non-0x20 byte)
    let unpadded = strip_padding(&padded);

    // Step 4: Gzip decompress
    let compressed_len = if unpadded.len() >= 4 {
        u32::from_be_bytes([unpadded[0], unpadded[1], unpadded[2], unpadded[3]]) as usize
    } else {
        return Err(CryptoError::CorruptedData);
    };

    if unpadded.len() < 4 + compressed_len {
        return Err(CryptoError::CorruptedData);
    }

    let decompressed = gzip_decompress(&unpadded[4..4 + compressed_len])?;

    // Step 5: Parse JSON, return data
    let wrapper: serde_json::Value = serde_json::from_slice(&decompressed)
        .map_err(|_| CryptoError::CorruptedData)?;

    wrapper.get("data")
        .cloned()
        .ok_or(CryptoError::CorruptedData)
}

// ─── Padding ─────────────────────────────────────────────────────────────────

fn pad(data: &[u8]) -> Vec<u8> {
    let block_size = PADDING_BLOCK_BYTES;
    let target_len = ((data.len() + block_size - 1) / block_size).max(1) * block_size;
    let mut result = vec![0x20u8; target_len];
    result[..data.len()].copy_from_slice(data);
    result
}

fn strip_padding(data: &[u8]) -> &[u8] {
    let mut end = data.len();
    while end > 0 && data[end - 1] == 0x20 {
        end -= 1;
    }
    &data[..end]
}

// ─── Gzip ────────────────────────────────────────────────────────────────────

fn gzip_compress(data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    // 4-byte big-endian length prefix + compressed data
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;
    let compressed = encoder.finish()
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;

    let mut result = Vec::with_capacity(4 + compressed.len());
    result.extend_from_slice(&(compressed.len() as u32).to_be_bytes());
    result.extend_from_slice(&compressed);
    Ok(result)
}

fn gzip_decompress(data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)
        .map_err(|_| CryptoError::CorruptedData)?;
    Ok(result)
}

// ─── Legacy functions (kept for reference, not used by new envelope format) ──

/// Generates a new X25519 keypair for key exchange (legacy, not used in current protocol).
pub fn generate_keypair_for_exchange() -> Option<([u8; 32], [u8; 32])> {
    let mut secret = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret);
    // x25519 clamping
    secret[0] &= 248;
    secret[31] &= 127;
    secret[31] |= 64;

    // Simple placeholder public key derivation (not real x25519)
    // This function is kept only for API compatibility; use x25519-dalek for real operations
    Some((secret, secret))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        let salt = b"test-salt-16byte";
        let key1 = derive_key("password123", salt);
        let key2 = derive_key("password123", salt);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let salt = b"test-salt-16byte";
        let key1 = derive_key("password123", salt);
        let key2 = derive_key("different", salt);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_envelope_roundtrip() {
        let salt = b"test-salt-16byte";
        let key = derive_key("test-password", salt);

        let original = serde_json::json!({
            "subscriptions": [{"id": "UC123", "name": "Channel"}],
            "playlists": [],
        });

        let envelope = encrypt_envelope(&original, &key, salt).unwrap();
        let decrypted = decrypt_envelope(&envelope, &key).unwrap();

        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_envelope_wrong_key_fails() {
        let salt = b"test-salt-16byte";
        let key1 = derive_key("password1", salt);
        let key2 = derive_key("password2", salt);

        let data = serde_json::json!({"test": true});
        let envelope = encrypt_envelope(&data, &key1, salt).unwrap();
        let result = decrypt_envelope(&envelope, &key2);

        assert!(result.is_err());
    }

    #[test]
    fn test_envelope_tampered_fails() {
        let salt = b"test-salt-16byte";
        let key = derive_key("test-password", salt);

        let data = serde_json::json!({"important": "data"});
        let mut envelope = encrypt_envelope(&data, &key, salt).unwrap();

        // Tamper with the ciphertext
        let bytes = unsafe { envelope.as_bytes_mut() };
        if bytes.len() > 50 {
            bytes[50] ^= 0xFF;
        }

        let result = decrypt_envelope(&envelope, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_padding_roundtrip() {
        let data = b"hello world";
        let padded = pad(data);
        assert!(padded.len() >= PADDING_BLOCK_BYTES);
        assert!(padded.len() % PADDING_BLOCK_BYTES == 0);
        assert_eq!(strip_padding(&padded), data);
    }

    #[test]
    fn test_gzip_roundtrip() {
        let original = b"Hello, this is a test of gzip compression and decompression!";
        let compressed = gzip_compress(original).unwrap();
        let decompressed = gzip_decompress(&compressed[4..]).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        assert_eq!(salt1.len(), 16);
        assert_eq!(salt2.len(), 16);
        assert_ne!(salt1, salt2);
    }

    #[test]
    fn test_import_key() {
        let raw = [42u8; 32];
        let key = import_key(&raw).unwrap();
        assert_eq!(key, raw);
    }

    #[test]
    fn test_import_key_wrong_length() {
        let raw = [42u8; 16];
        let result = import_key(&raw);
        assert!(result.is_err());
    }
}
