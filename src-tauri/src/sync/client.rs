use std::collections::HashMap;
use std::time::Duration;

use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::sync::crypto;
use crate::sync::models::*;

/// Minimum bytes per second for encrypted sync timeouts.
const ENCRYPTED_SYNC_MIN_BYTES_PER_SECOND: u64 = 128 * 1024;
/// Maximum timeout for encrypted sync uploads/downloads.
const ENCRYPTED_SYNC_MAX_TIMEOUT_SECS: u64 = 300;

/// Sync client for the OpenTubeX sync server protocol.
///
/// Supports:
/// - Health / capabilities detection (`GET /health`)
/// - Registration / login (`POST /account/{register,login}`)
/// - Account deletion (`DELETE /account/delete`)
/// - Encrypted sync manifest (`GET /v1/encrypted_sync`)
/// - Per-collection download/upload with optimistic concurrency
pub struct SyncClient {
    server_url: String,
    token: String,
    client: Client,
    api_prefix: Option<String>,
}

impl SyncClient {
    /// Creates a new sync client.
    ///
    /// `server_url` will be normalized: trailing slashes, `/v1`, and `/docs` are stripped.
    pub fn new(server_url: &str, token: &str) -> Result<Self, SyncError> {
        let server_url = Self::normalize_url(server_url);

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| SyncError::Network(e.to_string()))?;

        Ok(Self {
            server_url,
            token: token.to_string(),
            client,
            api_prefix: None,
        })
    }

    /// Creates a client from an existing account state.
    pub fn from_state(state: &SyncAccountState) -> Result<Self, SyncError> {
        Self::new(&state.server_url, &state.token)
    }

    // ─── URL helpers ───────────────────────────────────────────────────────

    fn normalize_url(url: &str) -> String {
        let mut url = url.trim_end_matches('/').to_string();
        // Strip trailing /v1 if present
        if url.ends_with("/v1") {
            url = url[..url.len() - 3].trim_end_matches('/').to_string();
        }
        // Strip trailing /docs if present
        if url.ends_with("/docs") {
            url = url[..url.len() - 5].trim_end_matches('/').to_string();
        }
        url
    }

    /// Returns the API prefix, auto-detecting `/v1` vs root on first call.
    async fn api_prefix(&mut self) -> Result<String, SyncError> {
        if let Some(prefix) = &self.api_prefix {
            return Ok(prefix.clone());
        }

        // Try /v1 first, fall back to root
        let health_v1 = self.try_endpoint("/v1/health").await;
        if health_v1 {
            self.api_prefix = Some("/v1".to_string());
            return Ok("/v1".to_string());
        }

        let health_root = self.try_endpoint("/health").await;
        if health_root {
            self.api_prefix = Some(String::new());
            return Ok(String::new());
        }

        // Default to /v1 if neither works
        self.api_prefix = Some("/v1".to_string());
        Ok("/v1".to_string())
    }

    /// Makes an unauthenticated GET request to check if an endpoint exists.
    async fn try_endpoint(&self, path: &str) -> bool {
        let url = format!("{}{}", self.server_url, path);
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success() || resp.status().as_u16() == 401,
            Err(_) => false,
        }
    }

    fn auth_header(&self) -> String {
        self.token.clone()
    }

    // ─── Health / Capabilities ──────────────────────────────────────────────

    /// Checks server health and detects capabilities.
    pub async fn health(&mut self) -> Result<HealthResponse, SyncError> {
        let prefix = self.api_prefix().await?;
        let url = format!("{}{}/health", self.server_url, prefix);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| SyncError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(SyncError::Server(format!(
                "Health check failed: HTTP {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| SyncError::Network(e.to_string()))?;

        // Old LibreTube servers return plain "OK"
        if body.trim() == "OK" {
            return Ok(HealthResponse {
                status: "ok".to_string(),
                capabilities: SyncCapabilities::default(),
            });
        }

        // Structured response
        let health: HealthResponse = serde_json::from_str(&body)
            .map_err(|e| SyncError::Server(format!("Invalid health response: {}", e)))?;

        Ok(health)
    }

    // ─── Auth ───────────────────────────────────────────────────────────────

    /// Registers a new account.
    pub async fn register(&mut self, username: &str, password: &str) -> Result<String, SyncError> {
        self.authenticate("register", username, password).await
    }

    /// Logs in to an existing account.
    pub async fn login(&mut self, username: &str, password: &str) -> Result<String, SyncError> {
        self.authenticate("login", username, password).await
    }

    async fn authenticate(
        &mut self,
        mode: &str,
        username: &str,
        password: &str,
    ) -> Result<String, SyncError> {
        let prefix = self.api_prefix().await?;
        let url = format!(
            "{}{}/account/{}",
            self.server_url, prefix, mode
        );

        let body = AuthRequest {
            name: username.to_string(),
            password: password.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SyncError::Network(e.to_string()))?;

        let status = response.status();
        let body_bytes = response.bytes().await
            .map_err(|e| SyncError::Network(e.to_string()))?;

        if status.as_u16() == 401 {
            return Err(SyncError::Auth("Invalid credentials".into()));
        }

        if !status.is_success() {
            let text = String::from_utf8_lossy(&body_bytes);
            return Err(SyncError::Server(format!(
                "Auth failed: HTTP {}: {}",
                status, text
            )));
        }

        let auth: AuthResponse = serde_json::from_slice(&body_bytes)
            .map_err(|e| SyncError::Server(format!("Invalid auth response: {}", e)))?;

        self.token = auth.jwt.clone();
        Ok(auth.jwt)
    }

    /// Deletes the account.
    pub async fn delete_account(&self, password: &str) -> Result<(), SyncError> {
        let prefix = self.api_prefix.clone().unwrap_or_default();
        let url = format!("{}{}/account/delete", self.server_url, prefix);

        #[derive(Serialize)]
        struct DeleteReq<'a> {
            password: &'a str,
        }

        let response = self
            .client
            .delete(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&DeleteReq { password })
            .send()
            .await
            .map_err(|e| SyncError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(SyncError::Server(format!(
                "Delete failed: HTTP {}",
                response.status()
            )));
        }

        Ok(())
    }

    // ─── Encrypted Sync ─────────────────────────────────────────────────────

    /// Fetches the encrypted sync manifest.
    pub async fn get_encrypted_sync_manifest(
        &self,
    ) -> Result<SyncManifest, SyncError> {
        let prefix = self.api_prefix.clone().unwrap_or_default();
        let url = format!("{}{}/encrypted_sync", self.server_url, prefix);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| SyncError::Network(e.to_string()))?;

        if response.status().as_u16() == 401 {
            return Err(SyncError::SessionExpired);
        }

        if !response.status().is_success() {
            return Err(SyncError::Server(format!(
                "Manifest fetch failed: HTTP {}",
                response.status()
            )));
        }

        let manifest: SyncManifest = response
            .json()
            .await
            .map_err(|e| SyncError::Server(format!("Invalid manifest: {}", e)))?;

        Ok(manifest)
    }

    /// Fetches a single encrypted collection.
    pub async fn get_encrypted_sync_collection(
        &self,
        collection: &str,
    ) -> Result<EncryptedSyncCollection, SyncError> {
        let prefix = self.api_prefix.clone().unwrap_or_default();
        let url = format!(
            "{}{}/encrypted_sync/{}",
            self.server_url, prefix, collection
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| SyncError::Network(e.to_string()))?;

        if response.status().as_u16() == 401 {
            return Err(SyncError::SessionExpired);
        }

        if response.status().as_u16() == 404 {
            // Collection doesn't exist yet
            return Ok(EncryptedSyncCollection {
                revision: 0,
                payload: None,
            });
        }

        if !response.status().is_success() {
            return Err(SyncError::Server(format!(
                "Collection fetch failed: HTTP {}",
                response.status()
            )));
        }

        let data: EncryptedSyncCollection = response
            .json()
            .await
            .map_err(|e| SyncError::Server(format!("Invalid collection: {}", e)))?;

        Ok(data)
    }

    /// Uploads an encrypted collection with optimistic concurrency.
    ///
    /// Returns `Err(SyncError::Conflict(_))` if the server rejects due to a stale revision (HTTP 409).
    pub async fn put_encrypted_sync_collection(
        &self,
        collection: &str,
        revision: i64,
        payload: &str,
    ) -> Result<(), SyncError> {
        let prefix = self.api_prefix.clone().unwrap_or_default();
        let url = format!(
            "{}{}/encrypted_sync/{}",
            self.server_url, prefix, collection
        );

        // Calculate timeout based on payload size
        let payload_bytes = payload.len() as u64;
        let timeout_secs = std::cmp::min(
            ENCRYPTED_SYNC_MAX_TIMEOUT_SECS,
            std::cmp::max(30, payload_bytes / ENCRYPTED_SYNC_MIN_BYTES_PER_SECOND),
        );

        let body = PutEncryptedSyncRequest {
            revision,
            payload: payload.to_string(),
        };

        let response = self
            .client
            .put(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(Duration::from_secs(timeout_secs))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    SyncError::Network("Request timed out".into())
                } else {
                    SyncError::Network(e.to_string())
                }
            })?;

        if response.status().as_u16() == 401 {
            return Err(SyncError::SessionExpired);
        }

        if response.status().as_u16() == 409 {
            return Err(SyncError::Conflict(collection.to_string()));
        }

        if !response.status().is_success() {
            return Err(SyncError::Server(format!(
                "Upload failed: HTTP {}",
                response.status()
            )));
        }

        Ok(())
    }
}

// ─── Crypto helpers on SyncClient ─────────────────────────────────────────────

/// Errors from key preparation.
#[derive(Debug, thiserror::Error)]
pub enum KeyPrepError {
    #[error("Invalid passphrase")]
    InvalidPassphrase,
    #[error("Crypto error: {0}")]
    Crypto(String),
}

/// Prepares the privacy key from a passphrase and salt.
///
/// If `existing_payload` is provided, validates the passphrase by attempting
/// to decrypt it. Returns the base64-encoded key and salt.
pub fn prepare_privacy_key(
    passphrase: &str,
    existing_payload: Option<&str>,
    existing_salt: Option<&str>,
) -> Result<(String, String), KeyPrepError> {
    let salt = match existing_salt {
        Some(s) => base64::engine::general_purpose::STANDARD.decode(s).map_err(|_| KeyPrepError::InvalidPassphrase)?,
        None => crypto::generate_salt().to_vec(),
    };

    let salt_arr: [u8; 16] = salt.as_slice().try_into()
        .map_err(|_| KeyPrepError::InvalidPassphrase)?;

    let key = crypto::derive_key(passphrase, &salt_arr);

    // If we have an existing payload, validate by decrypting
    if let Some(payload) = existing_payload {
        if !payload.is_empty() {
            crypto::decrypt_envelope(payload, &key)
                .map_err(|_| KeyPrepError::InvalidPassphrase)?;
        }
    }

    Ok((base64::engine::general_purpose::STANDARD.encode(&key), base64::engine::general_purpose::STANDARD.encode(&salt)))
}

/// Encrypts a sync document for upload.
pub fn encrypt_sync_document(
    data: &Value,
    key_b64: &str,
    salt_b64: &str,
) -> Result<String, KeyPrepError> {
    let key_bytes = base64::engine::general_purpose::STANDARD.decode(key_b64).map_err(|_| KeyPrepError::Crypto("Invalid key".into()))?;
    let key: [u8; 32] = key_bytes.try_into().map_err(|_| KeyPrepError::Crypto("Invalid key length".into()))?;

    let salt = base64::engine::general_purpose::STANDARD.decode(salt_b64).map_err(|_| KeyPrepError::Crypto("Invalid salt".into()))?;

    crypto::encrypt_envelope(data, &key, &salt)
        .map_err(|e| KeyPrepError::Crypto(e.to_string()))
}

/// Decrypts a sync document from download.
pub fn decrypt_sync_document(
    payload: &str,
    key_b64: &str,
) -> Result<Value, KeyPrepError> {
    let key_bytes = base64::engine::general_purpose::STANDARD.decode(key_b64).map_err(|_| KeyPrepError::Crypto("Invalid key".into()))?;
    let key: [u8; 32] = key_bytes.try_into().map_err(|_| KeyPrepError::Crypto("Invalid key length".into()))?;

    crypto::decrypt_envelope(payload, &key)
        .map_err(|e| KeyPrepError::Crypto(e.to_string()))
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url_trailing_slash() {
        assert_eq!(SyncClient::normalize_url("https://sync.example.com/"), "https://sync.example.com");
    }

    #[test]
    fn test_normalize_url_strips_v1() {
        assert_eq!(SyncClient::normalize_url("https://sync.example.com/v1"), "https://sync.example.com");
    }

    #[test]
    fn test_normalize_url_strips_docs() {
        assert_eq!(SyncClient::normalize_url("https://sync.example.com/docs"), "https://sync.example.com");
    }

    #[test]
    fn test_prepare_privacy_key_new_account() {
        let (key_b64, salt_b64) = prepare_privacy_key("my-passphrase", None, None).unwrap();
        assert!(!key_b64.is_empty());
        assert!(!salt_b64.is_empty());

        // Key should be 32 bytes → base64
        let key = base64::engine::general_purpose::STANDARD.decode(&key_b64).unwrap();
        assert_eq!(key.len(), 32);

        // Salt should be 16 bytes → base64
        let salt = base64::engine::general_purpose::STANDARD.decode(&salt_b64).unwrap();
        assert_eq!(salt.len(), 16);
    }

    #[test]
    fn test_prepare_privacy_key_with_existing_payload() {
        // First create a valid key + payload
        let (key_b64, salt_b64) = prepare_privacy_key("my-passphrase", None, None).unwrap();
        let data = serde_json::json!({"subscriptions": []});
        let payload = encrypt_sync_document(&data, &key_b64, &salt_b64).unwrap();

        // Should validate successfully with correct passphrase
        let result = prepare_privacy_key("my-passphrase", Some(&payload), Some(&salt_b64));
        assert!(result.is_ok());

        // Should fail with wrong passphrase
        let result = prepare_privacy_key("wrong-passphrase", Some(&payload), Some(&salt_b64));
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_decrypt_sync_document() {
        let (key_b64, salt_b64) = prepare_privacy_key("test-pass", None, None).unwrap();

        let original = serde_json::json!({
            "subscriptions": [
                {"id": "UC123", "name": "Channel A"},
                {"id": "UC456", "name": "Channel B"}
            ],
            "playlists": []
        });

        let payload = encrypt_sync_document(&original, &key_b64, &salt_b64).unwrap();
        let decrypted = decrypt_sync_document(&payload, &key_b64).unwrap();

        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_sync_privacy_mode_roundtrip() {
        let mode = SyncPrivacyMode::Enhanced;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"enhanced\"");

        let decoded: SyncPrivacyMode = serde_json::from_str(&json).unwrap();
        assert!(decoded.is_enhanced());
    }

    #[test]
    fn test_health_response_parsing_structured() {
        let json = r#"{"status":"ok","capabilities":{"encrypted_sync":1,"bulk_sync":1,"history_page_size":50}}"#;
        let health: HealthResponse = serde_json::from_str(json).unwrap();
        assert!(health.supports_encrypted_sync());
        assert!(health.supports_bulk_sync());
        assert_eq!(health.capabilities.history_page_size, 50);
    }

    #[test]
    fn test_sync_error_session_expired() {
        let err = SyncError::SessionExpired;
        assert!(err.is_session_expired());

        let err = SyncError::Auth("401 unauthorized".into());
        assert!(err.is_session_expired());
    }
}
