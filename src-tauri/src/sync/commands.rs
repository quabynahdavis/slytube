use std::collections::HashMap;
use std::sync::Mutex;

use tauri::State;
use uuid::Uuid;

use crate::sync::client::{
    decrypt_sync_document, encrypt_sync_document, prepare_privacy_key, SyncClient,
};
use crate::sync::models::*;

/// Shared state for tracking active sync operations.
pub struct SyncManager {
    /// Set of operation IDs that have been requested to cancel.
    pub cancel_tokens: Mutex<HashMap<String, bool>>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            cancel_tokens: Mutex::new(HashMap::new()),
        }
    }

    /// Requests cancellation of a sync operation.
    pub fn request_cancel(&self, operation_id: &str) {
        let mut tokens = self.cancel_tokens.lock().unwrap();
        tokens.insert(operation_id.to_string(), true);
    }

    /// Checks if an operation has been cancelled.
    pub fn is_cancelled(&self, operation_id: &str) -> bool {
        let tokens = self.cancel_tokens.lock().unwrap();
        tokens.get(operation_id).copied().unwrap_or(false)
    }

    /// Cleans up a cancel token after operation completes.
    pub fn clear_cancel(&self, operation_id: &str) {
        let mut tokens = self.cancel_tokens.lock().unwrap();
        tokens.remove(operation_id);
    }
}

// ─── Connection & Auth ────────────────────────────────────────────────────────

/// Tests connectivity to the sync server and returns server capabilities.
#[tauri::command]
pub async fn sync_test_connection(
    server_url: String,
) -> Result<HealthResponse, String> {
    let mut client = SyncClient::new(&server_url, "")
        .map_err(|e| e.to_string())?;

    client.health().await.map_err(|e| e.to_string())
}

/// Registers a new sync account.
#[tauri::command]
pub async fn sync_register(
    server_url: String,
    username: String,
    password: String,
) -> Result<String, String> {
    let mut client = SyncClient::new(&server_url, "")
        .map_err(|e| e.to_string())?;

    client
        .register(&username, &password)
        .await
        .map_err(|e| e.to_string())
}

/// Logs in to an existing sync account.
#[tauri::command]
pub async fn sync_login(
    server_url: String,
    username: String,
    password: String,
) -> Result<String, String> {
    let mut client = SyncClient::new(&server_url, "")
        .map_err(|e| e.to_string())?;

    client
        .login(&username, &password)
        .await
        .map_err(|e| e.to_string())
}

/// Deletes the sync account.
#[tauri::command]
pub async fn sync_delete_account(
    server_url: String,
    token: String,
    password: String,
) -> Result<(), String> {
    let client = SyncClient::new(&server_url, &token)
        .map_err(|e| e.to_string())?;

    client
        .delete_account(&password)
        .await
        .map_err(|e| e.to_string())
}

// ─── Encrypted Sync ───────────────────────────────────────────────────────────

/// Authenticates and prepares the privacy key for encrypted sync.
///
/// This derives the AES-GCM key from the passphrase and salt, validates
/// it against an existing payload if present, and returns the key and salt
/// for storage.
#[tauri::command]
pub async fn sync_prepare_key(
    passphrase: String,
    existing_payload: Option<String>,
    existing_salt: Option<String>,
) -> Result<(String, String), String> {
    prepare_privacy_key(&passphrase, existing_payload.as_deref(), existing_salt.as_deref())
        .map_err(|e| e.to_string())
}

/// Encrypts a sync document using the account's key.
#[tauri::command]
pub async fn sync_encrypt(
    data: serde_json::Value,
    key: String,
    salt: String,
) -> Result<String, String> {
    encrypt_sync_document(&data, &key, &salt).map_err(|e| e.to_string())
}

/// Decrypts a sync document using the account's key.
#[tauri::command]
pub async fn sync_decrypt(
    payload: String,
    key: String,
) -> Result<serde_json::Value, String> {
    decrypt_sync_document(&payload, &key).map_err(|e| e.to_string())
}

/// Fetches the encrypted sync manifest.
#[tauri::command]
pub async fn sync_get_manifest(
    server_url: String,
    token: String,
) -> Result<SyncManifest, String> {
    let client = SyncClient::new(&server_url, &token)
        .map_err(|e| e.to_string())?;

    // Use the manifest from the enhanced endpoint
    client.get_encrypted_sync_manifest().await.map_err(|e| {
        if e.is_session_expired() {
            "Session expired. Please log in again.".to_string()
        } else {
            e.to_string()
        }
    })
}

/// Fetches a single encrypted collection.
#[tauri::command]
pub async fn sync_get_collection(
    server_url: String,
    token: String,
    collection: String,
) -> Result<EncryptedSyncCollection, String> {
    let client = SyncClient::new(&server_url, &token)
        .map_err(|e| e.to_string())?;

    client
        .get_encrypted_sync_collection(&collection)
        .await
        .map_err(|e| {
            if e.is_session_expired() {
                "Session expired. Please log in again.".to_string()
            } else {
                e.to_string()
            }
        })
}

/// Uploads a single encrypted collection with optimistic concurrency.
#[tauri::command]
pub async fn sync_upload_collection(
    server_url: String,
    token: String,
    collection: String,
    revision: i64,
    payload: String,
) -> Result<(), String> {
    let client = SyncClient::new(&server_url, &token)
        .map_err(|e| e.to_string())?;

    // Retry up to 3 times on conflict (stale revision)
    let mut last_error = None;
    for attempt in 0..3 {
        match client
            .put_encrypted_sync_collection(&collection, revision, &payload)
            .await
        {
            Ok(()) => return Ok(()),
            Err(SyncError::Conflict(_)) => {
                last_error = Some("Conflict: stale revision. Retrying...".to_string());
                if attempt < 2 {
                    tokio::time::sleep(std::time::Duration::from_millis(500 * (attempt + 1))).await;
                }
            }
            Err(e) => return Err(e.to_string()),
        }
    }

    Err(last_error.unwrap_or_else(|| "Upload failed after retries".to_string()))
}

/// Returns the current sync state (stub — reads from settings in a full impl).
#[tauri::command]
pub async fn sync_get_state() -> Result<serde_json::Value, String> {
    // In a full implementation, load from db_settings
    Ok(serde_json::json!({
        "enabled": false,
        "serverUrl": "",
        "username": "",
        "privacyMode": "unknown",
    }))
}

/// Starts a full sync operation (simplified — full implementation needs DB integration).
#[tauri::command]
pub async fn sync_start(
    server_url: String,
    token: String,
    sync_manager: State<'_, SyncManager>,
) -> Result<SyncResult, String> {
    let operation_id = Uuid::new_v4().to_string();

    tracing::info!(
        "Starting sync operation {} for server {}",
        operation_id,
        server_url
    );

    let mut client = SyncClient::new(&server_url, &token)
        .map_err(|e| format!("Failed to create client: {}", e))?;

    // Check connection and capabilities
    let health = client
        .health()
        .await
        .map_err(|e| format!("Health check failed: {}", e))?;

    if sync_manager.is_cancelled(&operation_id) {
        sync_manager.clear_cancel(&operation_id);
        return Err("Sync cancelled".to_string());
    }

    // For encrypted sync, fetch the manifest
    let mut downloaded = Vec::new();
    let mut uploaded = Vec::new();

    if health.supports_encrypted_sync() {
        match client.get_encrypted_sync_manifest().await {
            Ok(manifest) => {
                for entry in &manifest.collections {
                    downloaded.push(entry.collection.clone());
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch manifest: {}", e);
            }
        }
    }

    sync_manager.clear_cancel(&operation_id);

    Ok(SyncResult {
        uploaded,
        downloaded,
        skipped: Vec::new(),
        errors: Vec::new(),
    })
}

/// Cancels any in-progress sync operation.
#[tauri::command]
pub async fn sync_cancel(sync_manager: State<'_, SyncManager>) -> Result<(), String> {
    let tokens = sync_manager.cancel_tokens.lock().unwrap();
    let active_ops: Vec<String> = tokens.keys().cloned().collect();
    drop(tokens);

    for op_id in &active_ops {
        sync_manager.request_cancel(op_id);
    }

    tracing::info!("Cancelled {} sync operation(s)", active_ops.len());
    Ok(())
}
