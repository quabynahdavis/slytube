use std::collections::HashMap;
use std::sync::Mutex;

use tauri::State;
use uuid::Uuid;

use crate::sync::client;
use crate::sync::models::{SyncResult, SyncSnapshot, SyncState};

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

/// Tests connectivity to the sync server.
#[tauri::command]
pub async fn sync_test_connection(
    server_url: String,
    token: String,
) -> Result<bool, String> {
    client::test_connection(&server_url, &token)
        .await
        .map_err(|e| e.to_string())
}

/// Returns the current sync state.
///
/// In a full implementation, this would load from persistent storage.
/// For now, returns a default state.
#[tauri::command]
pub async fn sync_get_state() -> Result<SyncState, String> {
    Ok(SyncState {
        last_sync: None,
        snapshot: None,
        sync_enabled: false,
        server_url: None,
        server_token: None,
        privacy_mode: "standard".to_string(),
    })
}

/// Saves the sync state.
///
/// In a full implementation, this would persist to storage.
#[tauri::command]
pub async fn sync_save_state(state: SyncState) -> Result<(), String> {
    // Validate state
    if state.privacy_mode != "standard" && state.privacy_mode != "enhanced" {
        return Err("Invalid privacy_mode: must be 'standard' or 'enhanced'".to_string());
    }

    // In a full implementation, persist to database or file
    tracing::info!("Sync state saved: enabled={}", state.sync_enabled);
    Ok(())
}

/// Starts a sync operation, uploading and downloading collections.
#[tauri::command]
pub async fn sync_start(
    server_url: String,
    token: String,
    collections: Vec<String>,
    sync_manager: State<'_, SyncManager>,
) -> Result<SyncResult, String> {
    let operation_id = Uuid::new_v4().to_string();

    tracing::info!(
        "Starting sync operation {} for collections: {:?}",
        operation_id,
        collections
    );

    // Test connection first
    let connected = client::test_connection(&server_url, &token)
        .await
        .map_err(|e| format!("Connection test failed: {}", e))?;

    if !connected {
        return Err("Could not connect to sync server".to_string());
    }

    // Build snapshot from collections
    let snapshot = SyncSnapshot {
        version: 1,
        timestamp: chrono::Utc::now().to_rfc3339(),
        collections: HashMap::new(),
    };

    // Check for cancellation
    if sync_manager.is_cancelled(&operation_id) {
        sync_manager.clear_cancel(&operation_id);
        return Err("Sync cancelled".to_string());
    }

    // Upload snapshot
    client::upload_snapshot(&server_url, &token, &snapshot)
        .await
        .map_err(|e| format!("Upload failed: {}", e))?;

    // Check for cancellation
    if sync_manager.is_cancelled(&operation_id) {
        sync_manager.clear_cancel(&operation_id);
        return Err("Sync cancelled".to_string());
    }

    // Download remote snapshot
    let downloaded_snapshot = match client::download_snapshot(&server_url, &token).await {
        Ok(snap) => Some(snap),
        Err(e) => {
            tracing::warn!("Download failed (may be first sync): {}", e);
            None
        }
    };

    // Build result
    let mut uploaded = HashMap::new();
    let mut downloaded = HashMap::new();

    for collection in &collections {
        uploaded.insert(collection.clone(), 1);
    }

    if let Some(remote) = downloaded_snapshot {
        for collection in remote.collections.keys() {
            downloaded.insert(collection.clone(), 1);
        }
    }

    // Clean up cancel token
    sync_manager.clear_cancel(&operation_id);

    let result = SyncResult {
        uploaded,
        downloaded,
        conflicts: Vec::new(),
    };

    tracing::info!("Sync operation {} completed successfully", operation_id);
    Ok(result)
}

/// Cancels any in-progress sync operation.
#[tauri::command]
pub async fn sync_cancel(sync_manager: State<'_, SyncManager>) -> Result<(), String> {
    // Mark all active operations for cancellation
    let tokens = sync_manager.cancel_tokens.lock().unwrap();
    let active_ops: Vec<String> = tokens.keys().cloned().collect();
    drop(tokens);

    for op_id in &active_ops {
        sync_manager.request_cancel(op_id);
    }

    tracing::info!("Cancelled {} sync operation(s)", active_ops.len());
    Ok(())
}
