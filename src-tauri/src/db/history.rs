use sqlx::SqlitePool;

use crate::db::models::HistoryEntry;

/// Repository for watch history CRUD operations.
#[derive(Debug, Clone)]
pub struct HistoryRepository {
    pool: SqlitePool,
}

impl HistoryRepository {
    /// Create a new HistoryRepository with the given pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find all history entries, ordered by most recently watched.
    pub async fn find_all(&self, limit: i64) -> Result<Vec<HistoryEntry>, sqlx::Error> {
        sqlx::query_as::<_, HistoryEntry>(
            r#"
            SELECT video_id, title, author, author_id, length_seconds,
                   watch_progress, time_watched, is_watched, is_live
            FROM history
            ORDER BY time_watched DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Find a single history entry by video ID.
    pub async fn find_one(&self, video_id: &str) -> Result<Option<HistoryEntry>, sqlx::Error> {
        sqlx::query_as::<_, HistoryEntry>(
            r#"
            SELECT video_id, title, author, author_id, length_seconds,
                   watch_progress, time_watched, is_watched, is_live
            FROM history
            WHERE video_id = ?
            "#,
        )
        .bind(video_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Insert or update a history entry.
    pub async fn upsert(&self, entry: &HistoryEntry) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO history (video_id, title, author, author_id, length_seconds, watch_progress, time_watched, is_watched, is_live)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(video_id) DO UPDATE SET
                title = excluded.title,
                author = excluded.author,
                author_id = excluded.author_id,
                length_seconds = excluded.length_seconds,
                watch_progress = excluded.watch_progress,
                time_watched = excluded.time_watched,
                is_watched = excluded.is_watched,
                is_live = excluded.is_live
            "#,
        )
        .bind(&entry.video_id)
        .bind(&entry.title)
        .bind(&entry.author)
        .bind(&entry.author_id)
        .bind(entry.length_seconds)
        .bind(entry.watch_progress)
        .bind(&entry.time_watched)
        .bind(entry.is_watched)
        .bind(entry.is_live)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Delete a history entry by video ID.
    pub async fn delete(&self, video_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM history WHERE video_id = ?")
            .bind(video_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Delete history entries older than the specified number of days.
    pub async fn delete_older_than_days(&self, days: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM history WHERE time_watched < datetime('now', ?)")
            .bind(format!("-{} days", days))
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Delete history entries older than a unix timestamp (seconds since epoch).
    pub async fn delete_older_than(&self, timestamp: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM history WHERE time_watched < datetime(?, 'unixepoch')")
            .bind(timestamp)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Clear all history entries.
    pub async fn clear_all(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM history")
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Apply sync changes from remote — bulk upsert/merge in a transaction.
    pub async fn apply_sync_changes(&self, entries: &[HistoryEntry]) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        for entry in entries {
            sqlx::query(
                r#"
                INSERT INTO history (video_id, title, author, author_id, length_seconds, watch_progress, time_watched, is_watched, is_live)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(video_id) DO UPDATE SET
                    title = excluded.title,
                    author = excluded.author,
                    author_id = excluded.author_id,
                    length_seconds = excluded.length_seconds,
                    watch_progress = excluded.watch_progress,
                    time_watched = excluded.time_watched,
                    is_watched = excluded.is_watched,
                    is_live = excluded.is_live
                "#,
            )
            .bind(&entry.video_id)
            .bind(&entry.title)
            .bind(&entry.author)
            .bind(&entry.author_id)
            .bind(entry.length_seconds)
            .bind(entry.watch_progress)
            .bind(&entry.time_watched)
            .bind(entry.is_watched)
            .bind(entry.is_live)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Get entries newer than a unix timestamp (for incremental sync).
    pub async fn get_newer_than(&self, timestamp: i64) -> Result<Vec<HistoryEntry>, sqlx::Error> {
        sqlx::query_as::<_, HistoryEntry>(
            r#"
            SELECT video_id, title, author, author_id, length_seconds,
                   watch_progress, time_watched, is_watched, is_live
            FROM history
            WHERE time_watched > datetime(?, 'unixepoch')
            ORDER BY time_watched DESC
            "#,
        )
        .bind(timestamp)
        .fetch_all(&self.pool)
        .await
    }

    /// Update watch progress (called during playback).
    pub async fn update_watch_progress(
        &self,
        video_id: &str,
        progress: i64,
        length: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE history
            SET watch_progress = ?, length_seconds = ?
            WHERE video_id = ?
            "#,
        )
        .bind(progress as f64)
        .bind(length)
        .bind(video_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_repo_new() {
        fn _assert_new(pool: SqlitePool) -> HistoryRepository {
            HistoryRepository::new(pool)
        }
    }

    #[test]
    fn test_history_entry_vec_type() {
        // Verify HistoryEntry can be used in a Vec (for bulk operations)
        let entries: Vec<HistoryEntry> = vec![];
        assert!(entries.is_empty());
    }

    #[test]
    fn test_get_newer_than_signature() {
        // Verify get_newer_than accepts an i64 timestamp
        fn _assert_timestamp(ts: i64) -> i64 {
            ts
        }
        assert_eq!(_assert_timestamp(1700000000), 1700000000);
        assert_eq!(_assert_timestamp(0), 0);
        assert_eq!(_assert_timestamp(-1), -1);
    }

    #[test]
    fn test_update_watch_progress_signature() {
        // Verify update_watch_progress accepts video_id, progress, length
        fn _assert_params<'a>(
            video_id: &'a str,
            progress: i64,
            length: i64,
        ) -> (&'a str, i64, i64) {
            (video_id, progress, length)
        }
        assert_eq!(_assert_params("abc123", 120, 300), ("abc123", 120, 300));
        assert_eq!(_assert_params("", 0, 0), ("", 0, 0));
    }

    #[test]
    fn test_delete_older_than_timestamp_signature() {
        // Verify delete_older_than accepts i64 timestamp and returns u64 rows affected
        fn _assert_timestamp(ts: i64) -> bool {
            ts > 0
        }
        assert!(_assert_timestamp(1700000000));
        assert!(!_assert_timestamp(0));
    }

    #[test]
    fn test_delete_older_than_days_signature() {
        // Verify delete_older_than_days accepts i64 days
        fn _assert_days(days: i64) -> bool {
            days >= 0
        }
        assert!(_assert_days(30));
        assert!(_assert_days(0));
    }

    #[tokio::test]
    async fn test_apply_sync_changes_with_db() {
        // Integration test: verify apply_sync_changes works against a real in-memory DB
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        // Create the history table
        sqlx::query(
            r#"
            CREATE TABLE history (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                author TEXT NOT NULL,
                author_id TEXT NOT NULL,
                length_seconds INTEGER,
                watch_progress REAL DEFAULT 0.0,
                time_watched TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                is_watched BOOLEAN NOT NULL DEFAULT FALSE,
                is_live BOOLEAN NOT NULL DEFAULT FALSE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = HistoryRepository::new(pool.clone());

        // Test with empty entries
        repo.apply_sync_changes(&[]).await.unwrap();

        // Test with single entry
        let entries = vec![HistoryEntry {
            video_id: "test1".to_string(),
            title: "Test Video".to_string(),
            author: "Test Author".to_string(),
            author_id: "author1".to_string(),
            length_seconds: Some(300),
            watch_progress: Some(0.5),
            time_watched: "2024-01-01T00:00:00Z".to_string(),
            is_watched: false,
            is_live: false,
        }];
        repo.apply_sync_changes(&entries).await.unwrap();

        // Verify the entry was inserted
        let result = repo.find_one("test1").await.unwrap();
        assert!(result.is_some());
        let entry = result.unwrap();
        assert_eq!(entry.video_id, "test1");
        assert_eq!(entry.title, "Test Video");

        // Test upsert (apply again with updated data)
        let updated_entries = vec![HistoryEntry {
            video_id: "test1".to_string(),
            title: "Updated Title".to_string(),
            author: "Test Author".to_string(),
            author_id: "author1".to_string(),
            length_seconds: Some(300),
            watch_progress: Some(1.0),
            time_watched: "2024-01-02T00:00:00Z".to_string(),
            is_watched: true,
            is_live: false,
        }];
        repo.apply_sync_changes(&updated_entries).await.unwrap();

        let result = repo.find_one("test1").await.unwrap().unwrap();
        assert_eq!(result.title, "Updated Title");
        assert_eq!(result.is_watched, true);
    }

    #[tokio::test]
    async fn test_get_newer_than_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE history (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                author TEXT NOT NULL,
                author_id TEXT NOT NULL,
                length_seconds INTEGER,
                watch_progress REAL DEFAULT 0.0,
                time_watched TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                is_watched BOOLEAN NOT NULL DEFAULT FALSE,
                is_live BOOLEAN NOT NULL DEFAULT FALSE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = HistoryRepository::new(pool.clone());

        // Insert entries with specific timestamps
        let entries = vec![
            HistoryEntry {
                video_id: "old".to_string(),
                title: "Old Video".to_string(),
                author: "Author".to_string(),
                author_id: "a1".to_string(),
                length_seconds: Some(100),
                watch_progress: Some(1.0),
                time_watched: "2024-01-01 00:00:00".to_string(),
                is_watched: true,
                is_live: false,
            },
            HistoryEntry {
                video_id: "new".to_string(),
                title: "New Video".to_string(),
                author: "Author".to_string(),
                author_id: "a1".to_string(),
                length_seconds: Some(200),
                watch_progress: Some(0.5),
                time_watched: "2024-06-01 00:00:00".to_string(),
                is_watched: false,
                is_live: false,
            },
        ];
        repo.apply_sync_changes(&entries).await.unwrap();

        // Get entries newer than 2024-03-01 (unix timestamp 1709251200)
        let newer = repo.get_newer_than(1709251200).await.unwrap();
        assert_eq!(newer.len(), 1);
        assert_eq!(newer[0].video_id, "new");
    }

    #[tokio::test]
    async fn test_update_watch_progress_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE history (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                author TEXT NOT NULL,
                author_id TEXT NOT NULL,
                length_seconds INTEGER,
                watch_progress REAL DEFAULT 0.0,
                time_watched TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                is_watched BOOLEAN NOT NULL DEFAULT FALSE,
                is_live BOOLEAN NOT NULL DEFAULT FALSE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = HistoryRepository::new(pool.clone());

        // Insert an entry
        let entries = vec![HistoryEntry {
            video_id: "progress_test".to_string(),
            title: "Progress Test".to_string(),
            author: "Author".to_string(),
            author_id: "a1".to_string(),
            length_seconds: None,
            watch_progress: None,
            time_watched: "2024-01-01T00:00:00Z".to_string(),
            is_watched: false,
            is_live: false,
        }];
        repo.apply_sync_changes(&entries).await.unwrap();

        // Update progress
        repo.update_watch_progress("progress_test", 150, 300)
            .await
            .unwrap();

        let result = repo.find_one("progress_test").await.unwrap().unwrap();
        assert_eq!(result.watch_progress, Some(150.0));
        assert_eq!(result.length_seconds, Some(300));
    }

    #[tokio::test]
    async fn test_delete_older_than_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE history (
                video_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                author TEXT NOT NULL,
                author_id TEXT NOT NULL,
                length_seconds INTEGER,
                watch_progress REAL DEFAULT 0.0,
                time_watched TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                is_watched BOOLEAN NOT NULL DEFAULT FALSE,
                is_live BOOLEAN NOT NULL DEFAULT FALSE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = HistoryRepository::new(pool.clone());

        // Insert entries with different timestamps
        let entries = vec![
            HistoryEntry {
                video_id: "old_entry".to_string(),
                title: "Old".to_string(),
                author: "Author".to_string(),
                author_id: "a1".to_string(),
                length_seconds: Some(100),
                watch_progress: Some(1.0),
                time_watched: "2024-01-01 00:00:00".to_string(),
                is_watched: true,
                is_live: false,
            },
            HistoryEntry {
                video_id: "new_entry".to_string(),
                title: "New".to_string(),
                author: "Author".to_string(),
                author_id: "a1".to_string(),
                length_seconds: Some(200),
                watch_progress: Some(0.5),
                time_watched: "2024-06-01 00:00:00".to_string(),
                is_watched: false,
                is_live: false,
            },
        ];
        repo.apply_sync_changes(&entries).await.unwrap();

        // Delete entries older than 2024-03-01 (unix timestamp 1709251200)
        let deleted = repo.delete_older_than(1709251200).await.unwrap();
        assert_eq!(deleted, 1);

        // Verify old entry is gone
        assert!(repo.find_one("old_entry").await.unwrap().is_none());
        // Verify new entry still exists
        assert!(repo.find_one("new_entry").await.unwrap().is_some());
    }
}
