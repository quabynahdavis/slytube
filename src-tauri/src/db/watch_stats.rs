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

    /// Add watch time for a video on today's date.
    pub async fn add_video_watch_time(&self, video_id: &str, seconds: f64) -> Result<(), sqlx::Error> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        // Try to update existing entry first (avoids needing ON CONFLICT with UNIQUE constraint)
        let result = sqlx::query(
            "UPDATE watch_stats SET watch_time = watch_time + ? WHERE video_id = ? AND date = ?",
        )
        .bind(seconds)
        .bind(video_id)
        .bind(&today)
        .execute(&self.pool)
        .await?;

        // If no rows were affected, insert a new entry
        if result.rows_affected() == 0 {
            sqlx::query(
                "INSERT INTO watch_stats (video_id, watch_time, date) VALUES (?, ?, ?)",
            )
            .bind(video_id)
            .bind(seconds)
            .bind(&today)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Add aggregated watch time for a specific date.
    /// Uses "_total" as video_id to distinguish from per-video stats.
    pub async fn add_watch_time(&self, date: &str, seconds: i64) -> Result<(), sqlx::Error> {
        // Try to update existing entry first (avoids needing ON CONFLICT with UNIQUE constraint)
        let result = sqlx::query(
            "UPDATE watch_stats SET watch_time = watch_time + ? WHERE video_id = '_total' AND date = ?",
        )
        .bind(seconds as f64)
        .bind(date)
        .execute(&self.pool)
        .await?;

        // If no rows were affected, insert a new entry
        if result.rows_affected() == 0 {
            sqlx::query(
                "INSERT INTO watch_stats (video_id, watch_time, date) VALUES (?, ?, ?)",
            )
            .bind("_total")
            .bind(seconds as f64)
            .bind(date)
            .execute(&self.pool)
            .await?;
        }

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

    /// Get aggregated watch stats for a specific date.
    pub async fn get_stats_for_date(&self, date: &str) -> Result<Option<WatchStat>, sqlx::Error> {
        sqlx::query_as::<_, WatchStat>(
            r#"
            SELECT 0 as id, '' as video_id, SUM(watch_time) as watch_time, date
            FROM watch_stats
            WHERE date = ?
            GROUP BY date
            "#,
        )
        .bind(date)
        .fetch_optional(&self.pool)
        .await
    }

    /// Get aggregated watch stats for a date range.
    pub async fn get_stats_range(
        &self,
        start: &str,
        end: &str,
    ) -> Result<Vec<WatchStat>, sqlx::Error> {
        sqlx::query_as::<_, WatchStat>(
            r#"
            SELECT 0 as id, '' as video_id, SUM(watch_time) as watch_time, date
            FROM watch_stats
            WHERE date >= ? AND date <= ?
            GROUP BY date
            ORDER BY date ASC
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
    }

    /// Migrate watch stats from history records (estimate from watch_progress * length_seconds).
    pub async fn migrate_from_history(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO watch_stats (video_id, watch_time, date)
            SELECT video_id,
                   COALESCE(length_seconds, 0) * COALESCE(watch_progress, 0.0) as watch_time,
                   date(time_watched) as date
            FROM history
            WHERE length_seconds IS NOT NULL AND watch_progress IS NOT NULL
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
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

    #[tokio::test]
    async fn test_add_watch_time_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE watch_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                video_id TEXT NOT NULL,
                watch_time REAL NOT NULL DEFAULT 0.0,
                date DATE NOT NULL DEFAULT CURRENT_DATE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = WatchStatsRepository::new(pool.clone());

        // Add watch time for a specific date
        repo.add_watch_time("2024-01-15", 300).await.unwrap();
        repo.add_watch_time("2024-01-15", 150).await.unwrap(); // Add more

        let stat = repo.get_stats_for_date("2024-01-15").await.unwrap();
        assert!(stat.is_some());
        let stat = stat.unwrap();
        assert_eq!(stat.watch_time, 450.0); // 300 + 150
    }

    #[tokio::test]
    async fn test_get_stats_for_date_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE watch_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                video_id TEXT NOT NULL,
                watch_time REAL NOT NULL DEFAULT 0.0,
                date DATE NOT NULL DEFAULT CURRENT_DATE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = WatchStatsRepository::new(pool.clone());

        // No data yet
        let stat = repo.get_stats_for_date("2024-01-15").await.unwrap();
        assert!(stat.is_none());

        // Add data
        repo.add_watch_time("2024-01-15", 500).await.unwrap();

        let stat = repo.get_stats_for_date("2024-01-15").await.unwrap();
        assert!(stat.is_some());
        assert_eq!(stat.unwrap().watch_time, 500.0);
    }

    #[tokio::test]
    async fn test_get_stats_range_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE watch_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                video_id TEXT NOT NULL,
                watch_time REAL NOT NULL DEFAULT 0.0,
                date DATE NOT NULL DEFAULT CURRENT_DATE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = WatchStatsRepository::new(pool.clone());

        // Add watch time for multiple dates
        repo.add_watch_time("2024-01-01", 100).await.unwrap();
        repo.add_watch_time("2024-01-02", 200).await.unwrap();
        repo.add_watch_time("2024-01-03", 300).await.unwrap();
        repo.add_watch_time("2024-01-04", 400).await.unwrap();

        // Get range
        let stats = repo
            .get_stats_range("2024-01-02", "2024-01-03")
            .await
            .unwrap();
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].watch_time, 200.0);
        assert_eq!(stats[1].watch_time, 300.0);
    }

    #[tokio::test]
    async fn test_migrate_from_history_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        // Create both tables
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

        sqlx::query(
            r#"
            CREATE TABLE watch_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                video_id TEXT NOT NULL,
                watch_time REAL NOT NULL DEFAULT 0.0,
                date DATE NOT NULL DEFAULT CURRENT_DATE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = WatchStatsRepository::new(pool.clone());

        // Insert history entries
        sqlx::query(
            r#"
            INSERT INTO history (video_id, title, author, author_id, length_seconds, watch_progress, time_watched, is_watched, is_live)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind("vid1")
        .bind("Video 1")
        .bind("Author")
        .bind("a1")
        .bind(300)
        .bind(0.5)
        .bind("2024-03-15 10:00:00")
        .bind(false)
        .bind(false)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            INSERT INTO history (video_id, title, author, author_id, length_seconds, watch_progress, time_watched, is_watched, is_live)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind("vid2")
        .bind("Video 2")
        .bind("Author")
        .bind("a1")
        .bind(600)
        .bind(1.0)
        .bind("2024-03-15 12:00:00")
        .bind(true)
        .bind(false)
        .execute(&pool)
        .await
        .unwrap();

        // Migrate
        repo.migrate_from_history().await.unwrap();

        // Verify: vid1 should have 300 * 0.5 = 150, vid2 should have 600 * 1.0 = 600
        let stat1 = repo.get_stats_for_video("vid1").await.unwrap();
        assert!(!stat1.is_empty());
        assert!((stat1[0].watch_time - 150.0).abs() < f64::EPSILON);

        let stat2 = repo.get_stats_for_video("vid2").await.unwrap();
        assert!(!stat2.is_empty());
        assert!((stat2[0].watch_time - 600.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_add_video_watch_time_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE watch_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                video_id TEXT NOT NULL,
                watch_time REAL NOT NULL DEFAULT 0.0,
                date DATE NOT NULL DEFAULT CURRENT_DATE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = WatchStatsRepository::new(pool.clone());

        // Add watch time for a video
        repo.add_video_watch_time("vid1", 120.5).await.unwrap();

        let stats = repo.get_stats_for_video("vid1").await.unwrap();
        assert!(!stats.is_empty());
        assert!((stats[0].watch_time - 120.5).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_get_total_watch_time_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE watch_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                video_id TEXT NOT NULL,
                watch_time REAL NOT NULL DEFAULT 0.0,
                date DATE NOT NULL DEFAULT CURRENT_DATE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = WatchStatsRepository::new(pool.clone());

        // Initially zero
        let total = repo.get_total_watch_time().await.unwrap();
        assert!((total - 0.0).abs() < f64::EPSILON);

        // Add some watch time
        repo.add_watch_time("2024-01-01", 100).await.unwrap();
        repo.add_watch_time("2024-01-02", 200).await.unwrap();

        let total = repo.get_total_watch_time().await.unwrap();
        assert!((total - 300.0).abs() < f64::EPSILON);
    }
}
