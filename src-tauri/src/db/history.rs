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
    pub async fn delete_older_than(&self, days: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM history WHERE time_watched < datetime('now', ?)")
            .bind(format!("-{} days", days))
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
}
