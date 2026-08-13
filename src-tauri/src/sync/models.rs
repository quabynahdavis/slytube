use serde::{Deserialize, Serialize};

// ─── Health / Capabilities ────────────────────────────────────────────────────

/// Response from `GET /health`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    #[serde(default)]
    pub capabilities: SyncCapabilities,
}

/// Server capability flags.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncCapabilities {
    /// Server supports encrypted sync (PBKDF2 + AES-GCM envelope).
    #[serde(default)]
    pub encrypted_sync: u8,
    /// Server supports bulk upload (chunked).
    #[serde(default)]
    pub bulk_sync: u8,
    /// History page size for paginated history fetches (fallback: 50).
    #[serde(default = "default_history_page_size")]
    pub history_page_size: u32,
}

fn default_history_page_size() -> u32 {
    50
}

impl HealthResponse {
    pub fn supports_encrypted_sync(&self) -> bool {
        self.capabilities.encrypted_sync == 1
    }

    pub fn supports_bulk_sync(&self) -> bool {
        self.capabilities.bulk_sync == 1
    }
}

// ─── Auth ─────────────────────────────────────────────────────────────────────

/// Response from `POST /account/{register|login}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub jwt: String,
}

/// Request body for auth endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub name: String,
    pub password: String,
}

// ─── Encrypted Sync Manifest ──────────────────────────────────────────────────

/// Response from `GET /v1/encrypted_sync`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncManifest {
    #[serde(default)]
    pub collections: Vec<ManifestEntry>,
    #[serde(default)]
    pub legacy_data: bool,
    #[serde(default)]
    pub legacy_encrypted_data: bool,
}

/// Single collection entry in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub collection: String,
    #[serde(default)]
    pub revision: i64,
}

// ─── Encrypted Sync Collection ────────────────────────────────────────────────

/// Response from `GET /v1/encrypted_sync/{collection}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSyncCollection {
    #[serde(default)]
    pub revision: i64,
    pub payload: Option<String>,
}

/// Request body for `PUT /v1/encrypted_sync/{collection}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutEncryptedSyncRequest {
    pub revision: i64,
    pub payload: String,
}

// ─── Encrypted Sync Document (decrypted contents) ─────────────────────────────

/// The decrypted contents of a sync document.
/// This is the `data` field extracted from the envelope.
pub type SyncDocument = serde_json::Value;

// ─── Collection names ─────────────────────────────────────────────────────────

/// The eight collections synced via encrypted sync.
pub const ENCRYPTED_COLLECTIONS: &[&str] = &[
    "subscriptions",
    "playlists",
    "history",
    "playbackSpeeds",
    "profiles",
    "sessions",
    "settings",
    "playlistBookmarks",
];

/// Collections that were previously stored as plaintext (for migration).
pub const LEGACY_ENCRYPTED_COLLECTIONS: &[&str] = &[
    "subscriptions",
    "playlists",
    "history",
    "playbackSpeeds",
    "profiles",
    "playlistBookmarks",
];

// ─── Account state ────────────────────────────────────────────────────────────

/// Persistent sync configuration and state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncAccountState {
    pub enabled: bool,
    pub server_url: String,
    pub username: String,
    pub token: String,
    pub privacy_mode: SyncPrivacyMode,
    /// Base64-encoded AES-GCM key (derived from passphrase + salt).
    pub privacy_key: String,
    /// Base64-encoded 16-byte KDF salt.
    pub privacy_salt: String,
    pub auto_sync: bool,
    pub last_sync_at: i64,
    /// JSON snapshot of last-synced collection data (for change detection).
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SyncPrivacyMode {
    Legacy,
    Enhanced,
    Unknown,
}

impl SyncPrivacyMode {
    pub fn is_enhanced(&self) -> bool {
        matches!(self, SyncPrivacyMode::Enhanced)
    }
}

// ─── Sync settings (user-toggles) ─────────────────────────────────────────────

/// Which collections the user has opted to sync.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncToggles {
    #[serde(default = "default_true")]
    pub subscriptions: bool,
    #[serde(default = "default_true")]
    pub playlists: bool,
    #[serde(default = "default_true")]
    pub history: bool,
    #[serde(default = "default_true")]
    pub playback_speeds: bool,
    #[serde(default = "default_true")]
    pub profiles: bool,
    #[serde(default = "default_true")]
    pub sessions: bool,
    #[serde(default = "default_true")]
    pub settings: bool,
}

fn default_true() -> bool {
    true
}

/// Which settings the user has opted out of syncing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncExcludedSettings {
    #[serde(default)]
    pub excluded: Vec<String>,
}

// ─── Sync result ──────────────────────────────────────────────────────────────

/// Result of a sync operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub uploaded: Vec<String>,
    pub downloaded: Vec<String>,
    pub skipped: Vec<String>,
    pub errors: Vec<String>,
}

/// Error types for sync operations.
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Server error: {0}")]
    Server(String),
    #[error("Crypto error: {0}")]
    Crypto(String),
    #[error("Conflict: stale revision for collection '{0}'")]
    Conflict(String),
    #[error("Data loss detected in collection '{0}' — {1}")]
    DataLoss(String, String),
    #[error("Session expired")]
    SessionExpired,
    #[error("Cancelled")]
    Cancelled,
}

impl SyncError {
    /// Returns true for errors that indicate an expired session (401).
    pub fn is_session_expired(&self) -> bool {
        matches!(self, SyncError::SessionExpired)
            || matches!(self, SyncError::Auth(msg) if msg.contains("401") || msg.contains("unauthorized"))
    }
}
