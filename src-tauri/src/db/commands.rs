use tauri::State;

use crate::db::models::{
    HistoryEntry, Playlist, PlaylistVideo, Profile, SearchEntry, Setting,
    SubscriptionCacheEntry, TabSession,
};
use crate::db::{
    DbPool, HistoryRepository, PlaylistsRepository, ProfilesRepository,
    SearchHistoryRepository, SettingsRepository, SubscriptionCacheRepository,
    TabSessionsRepository, WatchStatsRepository,
};

// =============================================================================
// Settings commands
// =============================================================================

#[tauri::command]
pub async fn db_settings_find_all(
    pool: State<'_, DbPool>,
) -> Result<Vec<Setting>, String> {
    let repo = SettingsRepository::new(pool.inner().cloned());
    repo.find_all().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_settings_find_one(
    pool: State<'_, DbPool>,
    id: String,
) -> Result<Option<Setting>, String> {
    let repo = SettingsRepository::new(pool.inner().cloned());
    repo.find_one(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_settings_upsert(
    pool: State<'_, DbPool>,
    id: String,
    value: String,
) -> Result<(), String> {
    let repo = SettingsRepository::new(pool.inner().cloned());
    repo.upsert(&id, &value).await.map_err(|e| e.to_string())
}

// =============================================================================
// Profiles commands
// =============================================================================

#[tauri::command]
pub async fn db_profiles_find_all(
    pool: State<'_, DbPool>,
) -> Result<Vec<Profile>, String> {
    let repo = ProfilesRepository::new(pool.inner().cloned());
    repo.find_all().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_profiles_find_one(
    pool: State<'_, DbPool>,
    id: String,
) -> Result<Option<Profile>, String> {
    let repo = ProfilesRepository::new(pool.inner().cloned());
    repo.find_one(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_profiles_create(
    pool: State<'_, DbPool>,
    profile: Profile,
) -> Result<(), String> {
    let repo = ProfilesRepository::new(pool.inner().cloned());
    repo.create(&profile).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_profiles_update(
    pool: State<'_, DbPool>,
    profile: Profile,
) -> Result<(), String> {
    let repo = ProfilesRepository::new(pool.inner().cloned());
    repo.update(&profile).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_profiles_delete(
    pool: State<'_, DbPool>,
    id: String,
) -> Result<(), String> {
    let repo = ProfilesRepository::new(pool.inner().cloned());
    repo.delete(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_profiles_get_subscriptions(
    pool: State<'_, DbPool>,
    profile_id: String,
) -> Result<Vec<String>, String> {
    let repo = ProfilesRepository::new(pool.inner().cloned());
    repo.get_subscriptions(&profile_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_profiles_add_subscription(
    pool: State<'_, DbPool>,
    profile_id: String,
    channel_id: String,
) -> Result<(), String> {
    let repo = ProfilesRepository::new(pool.inner().cloned());
    repo.add_subscription(&profile_id, &channel_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_profiles_remove_subscription(
    pool: State<'_, DbPool>,
    profile_id: String,
    channel_id: String,
) -> Result<(), String> {
    let repo = ProfilesRepository::new(pool.inner().cloned());
    repo.remove_subscription(&profile_id, &channel_id)
        .await
        .map_err(|e| e.to_string())
}

// =============================================================================
// Playlists commands
// =============================================================================

#[tauri::command]
pub async fn db_playlists_find_all(
    pool: State<'_, DbPool>,
    profile_id: String,
) -> Result<Vec<Playlist>, String> {
    let repo = PlaylistsRepository::new(pool.inner().cloned());
    repo.find_all(&profile_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_playlists_find_one(
    pool: State<'_, DbPool>,
    id: String,
) -> Result<Option<Playlist>, String> {
    let repo = PlaylistsRepository::new(pool.inner().cloned());
    repo.find_one(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_playlists_create(
    pool: State<'_, DbPool>,
    playlist: Playlist,
) -> Result<(), String> {
    let repo = PlaylistsRepository::new(pool.inner().cloned());
    repo.create(&playlist).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_playlists_update(
    pool: State<'_, DbPool>,
    playlist: Playlist,
) -> Result<(), String> {
    let repo = PlaylistsRepository::new(pool.inner().cloned());
    repo.update(&playlist).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_playlists_delete(
    pool: State<'_, DbPool>,
    id: String,
) -> Result<(), String> {
    let repo = PlaylistsRepository::new(pool.inner().cloned());
    repo.delete(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_playlists_get_videos(
    pool: State<'_, DbPool>,
    playlist_id: String,
) -> Result<Vec<PlaylistVideo>, String> {
    let repo = PlaylistsRepository::new(pool.inner().cloned());
    repo.get_videos(&playlist_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_playlists_add_video(
    pool: State<'_, DbPool>,
    playlist_id: String,
    video_id: String,
    position: i64,
) -> Result<(), String> {
    let repo = PlaylistsRepository::new(pool.inner().cloned());
    repo.add_video(&playlist_id, &video_id, position)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_playlists_remove_video(
    pool: State<'_, DbPool>,
    playlist_id: String,
    video_id: String,
) -> Result<(), String> {
    let repo = PlaylistsRepository::new(pool.inner().cloned());
    repo.remove_video(&playlist_id, &video_id)
        .await
        .map_err(|e| e.to_string())
}

// =============================================================================
// History commands
// =============================================================================

#[tauri::command]
pub async fn db_history_find_all(
    pool: State<'_, DbPool>,
    limit: i64,
) -> Result<Vec<HistoryEntry>, String> {
    let repo = HistoryRepository::new(pool.inner().cloned());
    repo.find_all(limit).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_history_find_one(
    pool: State<'_, DbPool>,
    video_id: String,
) -> Result<Option<HistoryEntry>, String> {
    let repo = HistoryRepository::new(pool.inner().cloned());
    repo.find_one(&video_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_history_upsert(
    pool: State<'_, DbPool>,
    entry: HistoryEntry,
) -> Result<(), String> {
    let repo = HistoryRepository::new(pool.inner().cloned());
    repo.upsert(&entry).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_history_delete(
    pool: State<'_, DbPool>,
    video_id: String,
) -> Result<(), String> {
    let repo = HistoryRepository::new(pool.inner().cloned());
    repo.delete(&video_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_history_clear(
    pool: State<'_, DbPool>,
) -> Result<u64, String> {
    let repo = HistoryRepository::new(pool.inner().cloned());
    repo.clear_all().await.map_err(|e| e.to_string())
}

// =============================================================================
// Watch stats commands
// =============================================================================

#[tauri::command]
pub async fn db_watch_stats_add(
    pool: State<'_, DbPool>,
    video_id: String,
    seconds: f64,
) -> Result<(), String> {
    let repo = WatchStatsRepository::new(pool.inner().cloned());
    repo.add_watch_time(&video_id, seconds)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_watch_stats_get_total(
    pool: State<'_, DbPool>,
) -> Result<f64, String> {
    let repo = WatchStatsRepository::new(pool.inner().cloned());
    repo.get_total_watch_time().await.map_err(|e| e.to_string())
}

// =============================================================================
// Search history commands
// =============================================================================

#[tauri::command]
pub async fn db_search_history_find_all(
    pool: State<'_, DbPool>,
    limit: i64,
) -> Result<Vec<SearchEntry>, String> {
    let repo = SearchHistoryRepository::new(pool.inner().cloned());
    repo.find_all(limit).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_search_history_add(
    pool: State<'_, DbPool>,
    query: String,
) -> Result<(), String> {
    let repo = SearchHistoryRepository::new(pool.inner().cloned());
    repo.add(&query).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_search_history_clear(
    pool: State<'_, DbPool>,
) -> Result<(), String> {
    let repo = SearchHistoryRepository::new(pool.inner().cloned());
    repo.clear_all().await.map_err(|e| e.to_string())
}

// =============================================================================
// Subscription cache commands
// =============================================================================

#[tauri::command]
pub async fn db_subscription_cache_find_one(
    pool: State<'_, DbPool>,
    channel_id: String,
) -> Result<Option<SubscriptionCacheEntry>, String> {
    let repo = SubscriptionCacheRepository::new(pool.inner().cloned());
    repo.find_one(&channel_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_subscription_cache_upsert(
    pool: State<'_, DbPool>,
    channel_id: String,
    data: String,
) -> Result<(), String> {
    let repo = SubscriptionCacheRepository::new(pool.inner().cloned());
    repo.upsert(&channel_id, &data)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_subscription_cache_get_all(
    pool: State<'_, DbPool>,
) -> Result<Vec<SubscriptionCacheEntry>, String> {
    let repo = SubscriptionCacheRepository::new(pool.inner().cloned());
    repo.get_all().await.map_err(|e| e.to_string())
}

// =============================================================================
// Tab sessions commands
// =============================================================================

#[tauri::command]
pub async fn db_tab_sessions_save(
    pool: State<'_, DbPool>,
    data: String,
) -> Result<i64, String> {
    let repo = TabSessionsRepository::new(pool.inner().cloned());
    repo.save(&data).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_tab_sessions_get_latest(
    pool: State<'_, DbPool>,
) -> Result<Option<TabSession>, String> {
    let repo = TabSessionsRepository::new(pool.inner().cloned());
    repo.get_latest().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_tab_sessions_clear(
    pool: State<'_, DbPool>,
) -> Result<(), String> {
    let repo = TabSessionsRepository::new(pool.inner().cloned());
    repo.clear_all().await.map_err(|e| e.to_string())
}
