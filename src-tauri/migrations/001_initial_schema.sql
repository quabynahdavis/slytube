-- Initial schema for SlyTube database
-- Migration 001: Create all tables and indexes

-- Settings table: key-value store for application settings
CREATE TABLE IF NOT EXISTS settings (
    id TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Profiles table: user profiles for multi-account support
CREATE TABLE IF NOT EXISTS profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    avatar TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Profile subscriptions: many-to-many relationship between profiles and channels
CREATE TABLE IF NOT EXISTS profile_subscriptions (
    profile_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    PRIMARY KEY (profile_id, channel_id),
    FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE
);

-- Playlists table: user-created playlists
CREATE TABLE IF NOT EXISTS playlists (
    id TEXT PRIMARY KEY,
    profile_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE
);

-- Playlist videos: videos within a playlist with ordering
CREATE TABLE IF NOT EXISTS playlist_videos (
    playlist_id TEXT NOT NULL,
    video_id TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (playlist_id, video_id),
    FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
);

-- History table: watched video history
CREATE TABLE IF NOT EXISTS history (
    video_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_id TEXT NOT NULL,
    length_seconds INTEGER,
    watch_progress REAL DEFAULT 0.0,
    time_watched TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_watched BOOLEAN NOT NULL DEFAULT FALSE,
    is_live BOOLEAN NOT NULL DEFAULT FALSE
);

-- Watch stats table: aggregated watch time statistics
CREATE TABLE IF NOT EXISTS watch_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    video_id TEXT NOT NULL,
    watch_time REAL NOT NULL DEFAULT 0.0,
    date DATE NOT NULL DEFAULT CURRENT_DATE
);

-- Search history table: past search queries
CREATE TABLE IF NOT EXISTS search_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Subscription cache table: cached subscription feed data
CREATE TABLE IF NOT EXISTS subscription_cache (
    channel_id TEXT PRIMARY KEY,
    data TEXT NOT NULL,
    last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Tab sessions table: saved tab states for session restoration
CREATE TABLE IF NOT EXISTS tab_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    data TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Download records table: track video downloads
CREATE TABLE IF NOT EXISTS download_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    video_id TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    percent REAL NOT NULL DEFAULT 0.0,
    destination TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Sync state table: store sync-related state information
CREATE TABLE IF NOT EXISTS sync_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- =============================================================================
-- Indexes for frequently queried columns
-- =============================================================================

-- Settings indexes
CREATE INDEX IF NOT EXISTS idx_settings_updated_at ON settings(updated_at);

-- Profile indexes
CREATE INDEX IF NOT EXISTS idx_profiles_created_at ON profiles(created_at);

-- Profile subscriptions indexes
CREATE INDEX IF NOT EXISTS idx_profile_subscriptions_channel_id ON profile_subscriptions(channel_id);

-- Playlist indexes
CREATE INDEX IF NOT EXISTS idx_playlists_profile_id ON playlists(profile_id);
CREATE INDEX IF NOT EXISTS idx_playlists_created_at ON playlists(created_at);

-- Playlist videos indexes
CREATE INDEX IF NOT EXISTS idx_playlist_videos_position ON playlist_videos(playlist_id, position);

-- History indexes
CREATE INDEX IF NOT EXISTS idx_history_time_watched ON history(time_watched DESC);
CREATE INDEX IF NOT EXISTS idx_history_author_id ON history(author_id);
CREATE INDEX IF NOT EXISTS idx_history_is_watched ON history(is_watched);

-- Watch stats indexes
CREATE INDEX IF NOT EXISTS idx_watch_stats_video_id ON watch_stats(video_id);
CREATE INDEX IF NOT EXISTS idx_watch_stats_date ON watch_stats(date);
CREATE INDEX IF NOT EXISTS idx_watch_stats_video_date ON watch_stats(video_id, date);

-- Search history indexes
CREATE INDEX IF NOT EXISTS idx_search_history_timestamp ON search_history(timestamp DESC);

-- Subscription cache indexes
CREATE INDEX IF NOT EXISTS idx_subscription_cache_last_updated ON subscription_cache(last_updated);

-- Download records indexes
CREATE INDEX IF NOT EXISTS idx_download_records_status ON download_records(status);
CREATE INDEX IF NOT EXISTS idx_download_records_created_at ON download_records(created_at DESC);

-- Tab sessions indexes
CREATE INDEX IF NOT EXISTS idx_tab_sessions_created_at ON tab_sessions(created_at DESC);
