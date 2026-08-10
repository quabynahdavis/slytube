use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
pub struct Setting {
    pub id: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
pub struct ProfileSubscription {
    pub profile_id: String,
    pub channel_id: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
pub struct Playlist {
    pub id: String,
    pub profile_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
pub struct PlaylistVideo {
    pub playlist_id: String,
    pub video_id: String,
    pub position: i64,
}

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
pub struct HistoryEntry {
    pub video_id: String,
    pub title: String,
    pub author: String,
    pub author_id: String,
    pub length_seconds: Option<i64>,
    pub watch_progress: Option<f64>,
    pub time_watched: String,
    pub is_watched: bool,
    pub is_live: bool,
}

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
pub struct WatchStat {
    pub id: i64,
    pub video_id: String,
    pub watch_time: f64,
    pub date: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
pub struct SearchEntry {
    pub id: i64,
    pub query: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionCacheEntry {
    pub channel_id: String,
    pub data: String,
    pub last_updated: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
pub struct TabSession {
    pub id: i64,
    pub data: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
pub struct DownloadRecord {
    pub id: i64,
    pub video_id: String,
    pub title: String,
    pub status: String,
    pub percent: f64,
    pub destination: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
pub struct SyncState {
    pub key: String,
    pub value: String,
}
