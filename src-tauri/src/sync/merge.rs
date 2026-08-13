use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::sync::models::SyncError;

/// Minimum number of deletions to trigger data-loss protection.
const DATA_LOSS_MIN_DELETIONS: usize = 10;
/// Maximum fraction of items that can be deleted without confirmation.
const DATA_LOSS_MAX_FRACTION: f64 = 0.5;

/// Result of merging two sets of IDs.
#[derive(Debug, Clone)]
pub struct MergeResult {
    pub merged: Vec<String>,
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

/// Merges local and remote ID sets with deletion-aware logic.
///
/// The merge rules (ported from OpenTubeX `mergeIds`):
/// 1. Union of all three sets (local, remote, previous).
/// 2. Items never seen before survive if present on either side.
/// 3. Items previously known survive only if present on BOTH local and remote.
///    → A deletion on either side wins once both sides have seen the item.
/// 4. If `allow_data_loss` is false and many items would be deleted,
///    returns `Err(DataLoss)`.
pub fn merge_ids(
    local_ids: &[String],
    remote_ids: &[String],
    previous_ids: &[String],
    allow_data_loss: bool,
    collection: &str,
) -> Result<MergeResult, SyncError> {
    let local_set: HashSet<_> = local_ids.iter().cloned().collect();
    let remote_set: HashSet<_> = remote_ids.iter().cloned().collect();
    let previous_set: HashSet<_> = previous_ids.iter().cloned().collect();

    let mut merged = HashSet::new();
    let mut added = Vec::new();
    let mut removed = Vec::new();

    // Union of all three sets
    let all: HashSet<_> = local_set
        .union(&remote_set)
        .cloned()
        .collect::<HashSet<_>>()
        .union(&previous_set)
        .cloned()
        .collect();

    for id in &all {
        let in_local = local_set.contains(id);
        let in_remote = remote_set.contains(id);
        let in_previous = previous_set.contains(id);

        if in_previous {
            if in_local && in_remote {
                // Present on both sides → keep
                merged.insert(id.clone());
            } else {
                // Missing from one or both sides → removed
                removed.push(id.clone());
            }
        } else {
            // Never seen before: survive if present on either side
            if in_local || in_remote {
                merged.insert(id.clone());
                added.push(id.clone());
            }
        }
    }

    // Data-loss guard
    if !allow_data_loss && !removed.is_empty() {
        let total = merged.len() + removed.len();
        if removed.len() >= DATA_LOSS_MIN_DELETIONS
            || (total > 0 && removed.len() as f64 / total as f64 > DATA_LOSS_MAX_FRACTION)
        {
            return Err(SyncError::DataLoss(
                collection.to_string(),
                format!(
                    "Would delete {} of {} previously synced items",
                    removed.len(),
                    total
                ),
            ));
        }
    }

    let mut merged_vec: Vec<_> = merged.into_iter().collect();
    merged_vec.sort();

    Ok(MergeResult {
        merged: merged_vec,
        added,
        removed,
    })
}

/// Subscription sync: merge channel subscriptions.
///
/// Returns the merged list of subscription IDs.
pub fn merge_subscriptions(
    local: &[String],
    remote: &[String],
    previous: &[String],
    allow_data_loss: bool,
) -> Result<MergeResult, SyncError> {
    merge_ids(local, remote, previous, allow_data_loss, "subscriptions")
}

/// History sync: merge watched video IDs.
///
/// Local entry wins when timestamps are equal or local is newer.
pub fn merge_history(
    local: &[HistoryEntry],
    remote: &[HistoryEntry],
    previous: &[String],
    allow_data_loss: bool,
) -> Result<Vec<HistoryEntry>, SyncError> {
    let local_ids: Vec<String> = local.iter().map(|e| e.video_id.clone()).collect();
    let remote_ids: Vec<String> = remote.iter().map(|e| e.video_id.clone()).collect();

    let merge_result = merge_ids(&local_ids, &remote_ids, previous, allow_data_loss, "history")?;

    // Build a map of video_id → entry for both sides
    let local_map: std::collections::HashMap<_, _> =
        local.iter().map(|e| (e.video_id.clone(), e)).collect();
    let remote_map: std::collections::HashMap<_, _> =
        remote.iter().map(|e| (e.video_id.clone(), e)).collect();

    // Merge entries: local wins when timestamps are equal or newer
    let mut merged = Vec::new();
    for id in &merge_result.merged {
        match (local_map.get(id), remote_map.get(id)) {
            (Some(l), Some(r)) => {
                if l.time_watched >= r.time_watched {
                    merged.push((*l).clone());
                } else {
                    merged.push((*r).clone());
                }
            }
            (Some(l), None) => merged.push((*l).clone()),
            (None, Some(r)) => merged.push((*r).clone()),
            (None, None) => {}
        }
    }

    Ok(merged)
}

/// Playlist sync: merge playlist metadata and per-playlist videos.
pub fn merge_playlists(
    local: &[PlaylistSync],
    remote: &[PlaylistSync],
    previous: &[PlaylistSync],
    allow_data_loss: bool,
) -> Result<Vec<PlaylistSync>, SyncError> {
    let local_ids: Vec<String> = local.iter().map(|p| p.id.clone()).collect();
    let remote_ids: Vec<String> = remote.iter().map(|p| p.id.clone()).collect();
    let previous_ids: Vec<String> = previous.iter().map(|p| p.id.clone()).collect();

    let merge_result = merge_ids(&local_ids, &remote_ids, &previous_ids, allow_data_loss, "playlists")?;

    let local_map: std::collections::HashMap<_, _> =
        local.iter().map(|p| (p.id.clone(), p)).collect();
    let remote_map: std::collections::HashMap<_, _> =
        remote.iter().map(|p| (p.id.clone(), p)).collect();

    let mut merged = Vec::new();
    for id in &merge_result.merged {
        match (local_map.get(id), remote_map.get(id)) {
            (Some(l), Some(r)) => {
                // Remote wins if remote changed and local didn't (LWW by updated_at)
                if r.last_updated > l.last_updated {
                    let mut playlist = (*r).clone();
                    // Merge video IDs within the playlist
                    playlist.videos = merge_playlist_videos(&l.videos, &r.videos);
                    merged.push(playlist);
                } else {
                    let mut playlist = (*l).clone();
                    playlist.videos = merge_playlist_videos(&l.videos, &r.videos);
                    merged.push(playlist);
                }
            }
            (Some(l), None) => merged.push((*l).clone()),
            (None, Some(r)) => merged.push((*r).clone()),
            (None, None) => {}
        }
    }

    Ok(merged)
}

/// Merges video IDs within a playlist.
fn merge_playlist_videos(local: &[String], remote: &[String]) -> Vec<String> {
    let local_set: HashSet<_> = local.iter().cloned().collect();
    let remote_set: HashSet<_> = remote.iter().cloned().collect();

    // Union, preserving order (local first, then remote-only)
    let mut result = local.to_vec();
    for id in remote {
        if !local_set.contains(id) {
            result.push(id.clone());
        }
    }
    result
}

/// Profile sync: merge subscription groups.
pub fn merge_profiles(
    local: &[ProfileSync],
    remote: &[ProfileSync],
    previous: &[ProfileSync],
    allow_data_loss: bool,
) -> Result<Vec<ProfileSync>, SyncError> {
    let local_ids: Vec<String> = local.iter().map(|p| p.id.clone()).collect();
    let remote_ids: Vec<String> = remote.iter().map(|p| p.id.clone()).collect();
    let previous_ids: Vec<String> = previous.iter().map(|p| p.id.clone()).collect();

    let merge_result = merge_ids(&local_ids, &remote_ids, &previous_ids, allow_data_loss, "profiles")?;

    let local_map: std::collections::HashMap<_, _> =
        local.iter().map(|p| (p.id.clone(), p)).collect();
    let remote_map: std::collections::HashMap<_, _> =
        remote.iter().map(|p| (p.id.clone(), p)).collect();

    let mut merged = Vec::new();
    for id in &merge_result.merged {
        match (local_map.get(id), remote_map.get(id)) {
            (Some(l), Some(r)) => {
                // Merge channels within the profile
                let channels = merge_profile_channels(&l.channels, &r.channels);
                let mut profile = (*l).clone();
                profile.channels = channels;
                if r.last_updated > l.last_updated {
                    profile.name = r.name.clone();
                    profile.last_updated = r.last_updated;
                }
                merged.push(profile);
            }
            (Some(l), None) => merged.push((*l).clone()),
            (None, Some(r)) => merged.push((*r).clone()),
            (None, None) => {}
        }
    }

    Ok(merged)
}

/// Merges channel IDs within a profile.
fn merge_profile_channels(local: &[String], remote: &[String]) -> Vec<String> {
    let local_set: HashSet<_> = local.iter().cloned().collect();
    let remote_set: HashSet<_> = remote.iter().cloned().collect();

    // Deletion-aware merge: union minus items deleted on either side
    let all: HashSet<_> = local_set.union(&remote_set).cloned().collect();
    let both: HashSet<_> = local_set.intersection(&remote_set).cloned().collect();

    let mut result: Vec<_> = all.into_iter().collect();
    // Prioritize items present on both sides
    result.sort_by_key(|id| if both.contains(id) { 0 } else { 1 });
    result
}

/// Setting sync: last-writer-wins by updated_at timestamp.
pub fn merge_settings(
    local: &[(String, Value, i64)],
    remote: &[(String, Value, i64)],
) -> Vec<(String, Value, i64)> {
    let mut map: std::collections::HashMap<String, (Value, i64)> = std::collections::HashMap::new();

    for (key, value, updated_at) in local {
        map.insert(key.clone(), (value.clone(), *updated_at));
    }

    for (key, value, updated_at) in remote {
        match map.get(key) {
            Some((_, existing_updated)) if *updated_at > *existing_updated => {
                map.insert(key.clone(), (value.clone(), *updated_at));
            }
            None => {
                map.insert(key.clone(), (value.clone(), *updated_at));
            }
            _ => {}
        }
    }

    let mut result: Vec<_> = map
        .into_iter()
        .map(|(key, (value, updated_at))| (key, value, updated_at))
        .collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

// ─── Sync data types for merge functions ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub video_id: String,
    pub title: String,
    pub author: String,
    pub author_id: String,
    pub time_watched: i64,
    pub watch_progress: i64,
    pub length_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistSync {
    pub id: String,
    pub title: String,
    pub description: String,
    pub videos: Vec<String>,
    pub last_updated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSync {
    pub id: String,
    pub name: String,
    pub channels: Vec<String>,
    pub last_updated: i64,
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_ids_basic_union() {
        let local = vec!["a".to_string(), "b".to_string()];
        let remote = vec!["b".to_string(), "c".to_string()];
        let previous: Vec<String> = vec![];

        let result = merge_ids(&local, &remote, &previous, false, "test").unwrap();
        let mut merged = result.merged.clone();
        merged.sort();
        assert_eq!(merged, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_merge_ids_deletion_wins() {
        let local = vec!["a".to_string(), "b".to_string()];
        let remote = vec!["b".to_string(), "c".to_string()];
        let previous = vec!["a".to_string(), "b".to_string(), "d".to_string()];

        // "d" was in previous but not in local or remote → deleted
        let result = merge_ids(&local, &remote, &previous, true, "test").unwrap();
        assert!(!result.merged.contains(&"d".to_string()));
        assert!(result.removed.contains(&"d".to_string()));
    }

    #[test]
    fn test_merge_ids_new_items_survive() {
        let local = vec!["a".to_string()];
        let remote = vec!["b".to_string()];
        let previous = vec!["a".to_string()];

        // "b" is new (not in previous) and in remote → survives
        let result = merge_ids(&local, &remote, &previous, false, "test").unwrap();
        assert!(result.merged.contains(&"b".to_string()));
        assert!(result.added.contains(&"b".to_string()));
    }

    #[test]
    fn test_merge_ids_data_loss_guard() {
        let local: Vec<String> = vec![];
        let remote: Vec<String> = vec![];
        let previous: Vec<String> = (0..20).map(|i| format!("item_{}", i)).collect();

        // Would delete 20 items → data loss guard triggers
        let result = merge_ids(&local, &remote, &previous, false, "test");
        assert!(matches!(result, Err(SyncError::DataLoss(_, _))));
    }

    #[test]
    fn test_merge_ids_data_loss_allowed() {
        let local: Vec<String> = vec![];
        let remote: Vec<String> = vec![];
        let previous: Vec<String> = (0..20).map(|i| format!("item_{}", i)).collect();

        // With allow_data_loss=true → succeeds
        let result = merge_ids(&local, &remote, &previous, true, "test").unwrap();
        assert_eq!(result.merged.len(), 0);
        assert_eq!(result.removed.len(), 20);
    }

    #[test]
    fn test_merge_subscriptions() {
        let local = vec!["UC1".to_string(), "UC2".to_string()];
        let remote = vec!["UC2".to_string(), "UC3".to_string()];
        let previous = vec!["UC1".to_string(), "UC2".to_string()];

        let result = merge_subscriptions(&local, &remote, &previous, false).unwrap();
        assert!(result.merged.contains(&"UC3".to_string()));
    }

    #[test]
    fn test_merge_history_local_wins() {
        let local = vec![HistoryEntry {
            video_id: "vid1".to_string(),
            title: "Video".to_string(),
            author: "Author".to_string(),
            author_id: "UC1".to_string(),
            time_watched: 100,
            watch_progress: 50,
            length_seconds: 120,
        }];
        let remote = vec![HistoryEntry {
            video_id: "vid1".to_string(),
            title: "Video".to_string(),
            author: "Author".to_string(),
            author_id: "UC1".to_string(),
            time_watched: 50,
            watch_progress: 30,
            length_seconds: 120,
        }];
        let previous = vec!["vid1".to_string()];

        let result = merge_history(&local, &remote, &previous, false).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].time_watched, 100); // Local wins (newer timestamp)
    }

    #[test]
    fn test_merge_settings_lww() {
        let local = vec![
            ("theme".to_string(), serde_json::json!("dark"), 100),
            ("volume".to_string(), serde_json::json!(80), 100),
        ];
        let remote = vec![
            ("theme".to_string(), serde_json::json!("light"), 150),
            ("autoplay".to_string(), serde_json::json!(true), 120),
        ];

        let result = merge_settings(&local, &remote);
        let map: std::collections::HashMap<_, _> = result.into_iter().map(|(k, v, _)| (k, v)).collect();

        // theme: remote is newer (150 > 100) → "light"
        assert_eq!(map.get("theme").unwrap(), &serde_json::json!("light"));
        // volume: only in local → 80
        assert_eq!(map.get("volume").unwrap(), &serde_json::json!(80));
        // autoplay: only in remote → true
        assert_eq!(map.get("autoplay").unwrap(), &serde_json::json!(true));
    }

    #[test]
    fn test_merge_playlists_video_union() {
        let videos = merge_playlist_videos(
            &["vid1".to_string(), "vid2".to_string()],
            &["vid2".to_string(), "vid3".to_string()],
        );
        assert_eq!(videos.len(), 3);
        assert!(videos.contains(&"vid1".to_string()));
        assert!(videos.contains(&"vid2".to_string()));
        assert!(videos.contains(&"vid3".to_string()));
    }

    #[test]
    fn test_merge_profile_channels() {
        let channels = merge_profile_channels(
            &["UC1".to_string(), "UC2".to_string()],
            &["UC2".to_string(), "UC3".to_string()],
        );
        assert!(channels.contains(&"UC1".to_string()));
        assert!(channels.contains(&"UC2".to_string()));
        assert!(channels.contains(&"UC3".to_string()));
    }
}
