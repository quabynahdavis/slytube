use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use hkdf::Hkdf;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use x25519_dalek::{PublicKey, EphemeralSecret};

/// Errors that can occur during cryptographic operations.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionError(String),
    #[error("Decryption failed: {0}")]
    DecryptionError(String),
    #[error("Invalid ciphertext format")]
    InvalidFormat,
    #[error("Key derivation failed: {0}")]
    KeyDerivationError(String),
}

/// Derives a 256-bit key from a password using PBKDF2 followed by HKDF.
///
/// # Arguments
/// * `password` - The user's password
/// * `salt` - Random salt bytes (should be at least 16 bytes)
pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    // First, use PBKDF2 to derive a key from the password
    let mut pbkdf2_key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut pbkdf2_key);

    // Then, use HKDF to derive the final key
    let hkdf = Hkdf::<Sha256>::new(Some(salt), &pbkdf2_key);
    let mut final_key = [0u8; 32];
    hkdf.expand(b"slytube-sync-v1", &mut final_key)
        .expect("HKDF expansion failed");

    final_key
}

/// Encrypts plaintext using AES-256-GCM with a random 96-bit nonce.
///
/// Returns the concatenation of: [nonce (12 bytes) || ciphertext]
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    // Generate random 96-bit nonce
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;

    // Serialize: nonce || ciphertext
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypts ciphertext using AES-256-GCM.
///
/// Expects the format: [nonce (12 bytes) || ciphertext]
pub fn decrypt(key: &[u8; 32], ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if ciphertext.len() < 12 {
        return Err(CryptoError::InvalidFormat);
    }

    let (nonce_bytes, encrypted) = ciphertext.split_at(12);

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, encrypted)
        .map_err(|e| CryptoError::DecryptionError(e.to_string()))
}

/// Generates a new X25519 keypair for key exchange.
pub fn generate_keypair() -> (PublicKey, EphemeralSecret) {
    let secret = EphemeralSecret::random_from_rng(rand::thread_rng());
    let public = PublicKey::from(&secret);
    (public, secret)
}

/// Serializes an encryption envelope for transmission.
///
/// Format: [nonce_length (1 byte) || nonce || ciphertext]
pub fn serialize_envelope(ciphertext: &[u8], nonce: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(1 + nonce.len() + ciphertext.len());
    result.push(nonce.len() as u8);
    result.extend_from_slice(nonce);
    result.extend_from_slice(ciphertext);
    result
}

/// Deserializes an encryption envelope.
///
/// Returns (ciphertext, nonce)
pub fn deserialize_envelope(data: &[u8]) -> (Vec<u8>, Vec<u8>) {
    if data.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let nonce_len = data[0] as usize;
    let nonce = data[1..1 + nonce_len].to_vec();
    let ciphertext = data[1 + nonce_len..].to_vec();

    (ciphertext, nonce)
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
    fn test_encrypt_decrypt_roundtrip() {
        let key = derive_key("test-password", b"test-salt-16byte");
        let plaintext = b"Hello, World! This is a secret message.";

        let ciphertext = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &ciphertext).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertexts() {
        let key = derive_key("test-password", b"test-salt-16byte");
        let plaintext = b"Same message";

        let ct1 = encrypt(&key, plaintext).unwrap();
        let ct2 = encrypt(&key, plaintext).unwrap();

        // Nonces are random, so ciphertexts should differ
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key1 = derive_key("password1", b"test-salt-16byte");
        let key2 = derive_key("password2", b"test-salt-16byte");
        let plaintext = b"Secret data";

        let ciphertext = encrypt(&key1, plaintext).unwrap();
        let result = decrypt(&key2, &ciphertext);

        assert!(result.is_err());
    }

    #[test]
    fn test_generate_keypair() {
        let (pub1, sec1) = generate_keypair();
        let (pub2, sec2) = generate_keypair();

        // Different keypairs should be generated
        assert_ne!(pub1.as_bytes(), pub2.as_bytes());
        assert_ne!(sec1.as_bytes(), sec2.as_bytes());
    }

    #[test]
    fn test_serialize_deserialize_envelope() {
        let ciphertext = vec![1, 2, 3, 4, 5];
        let nonce = vec![6, 7, 8, 9, 10, 11, 12];

        let envelope = serialize_envelope(&ciphertext, &nonce);
        let (dec, den) = deserialize_envelope(&envelope);

        assert_eq!(dec, ciphertext);
        assert_eq!(den, nonce);
    }
}
