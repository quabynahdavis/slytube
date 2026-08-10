use chrono::Utc;
use sqlx::SqlitePool;

use crate::db::models::WatchStat;

/// Repository for watch statistics CRUD operations.
#[derive(Debug, Clone)]
pub struct WatchStatsRepository {
    pool: SqlitePool,
}

impl WatchStatsRepository {
    /// Create a new WatchStatsRepository with the given pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Add watch time for a video.
    pub async fn add_watch_time(&self, video_id: &str, seconds: f64) -> Result<(), sqlx::Error> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        sqlx::query(
            r#"
            INSERT INTO watch_stats (video_id, watch_time, date)
            VALUES (?, ?, ?)
            ON CONFLICT(video_id, date) DO UPDATE SET
                watch_time = watch_time + excluded.watch_time
            "#,
        )
        .bind(video_id)
        .bind(seconds)
        .bind(&today)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get total watch time across all videos.
    pub async fn get_total_watch_time(&self) -> Result<f64, sqlx::Error> {
        let total: Option<f64> = sqlx::query_scalar("SELECT SUM(watch_time) FROM watch_stats")
            .fetch_one(&self.pool)
            .await?;
        Ok(total.unwrap_or(0.0))
    }

    /// Get watch time grouped by date for the last N days.
    pub async fn get_watch_time_by_date(
        &self,
        days: i64,
    ) -> Result<Vec<(String, f64)>, sqlx::Error> {
        sqlx::query_as::<_, (String, f64)>(
            r#"
            SELECT date, SUM(watch_time) as total
            FROM watch_stats
            WHERE date >= date('now', ?)
            GROUP BY date
            ORDER BY date ASC
            "#,
        )
        .bind(format!("-{} days", days))
        .fetch_all(&self.pool)
        .await
    }

    /// Get all watch stats for a specific video.
    pub async fn get_stats_for_video(&self, video_id: &str) -> Result<Vec<WatchStat>, sqlx::Error> {
        sqlx::query_as::<_, WatchStat>(
            "SELECT id, video_id, watch_time, date FROM watch_stats WHERE video_id = ? ORDER BY date DESC",
        )
        .bind(video_id)
        .fetch_all(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_stats_repo_new() {
        fn _assert_new(pool: SqlitePool) -> WatchStatsRepository {
            WatchStatsRepository::new(pool)
        }
    }
}
