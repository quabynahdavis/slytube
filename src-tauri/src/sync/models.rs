use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Represents a point-in-time snapshot of user data for syncing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSnapshot {
    pub version: u32,
    pub timestamp: String,
    pub collections: HashMap<String, serde_json::Value>,
}

/// Result of a sync operation, detailing what was uploaded/downloaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub uploaded: HashMap<String, usize>,
    pub downloaded: HashMap<String, usize>,
    pub conflicts: Vec<String>,
}

/// Persistent state for the sync system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub last_sync: Option<String>,
    pub snapshot: Option<SyncSnapshot>,
    pub sync_enabled: bool,
    pub server_url: Option<String>,
    pub server_token: Option<String>,
    pub privacy_mode: String, // "standard" or "enhanced"
}
