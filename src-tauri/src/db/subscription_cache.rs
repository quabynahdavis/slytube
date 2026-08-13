use chrono::Utc;
use sqlx::SqlitePool;

use crate::db::models::SubscriptionCacheEntry;

/// Repository for subscription cache CRUD operations.
#[derive(Debug, Clone)]
pub struct SubscriptionCacheRepository {
    pool: SqlitePool,
}

impl SubscriptionCacheRepository {
    /// Create a new SubscriptionCacheRepository with the given pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find a single cache entry by channel ID.
    pub async fn find_one(
        &self,
        channel_id: &str,
    ) -> Result<Option<SubscriptionCacheEntry>, sqlx::Error> {
        sqlx::query_as::<_, SubscriptionCacheEntry>(
            "SELECT channel_id, data, last_updated FROM subscription_cache WHERE channel_id = ?",
        )
        .bind(channel_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Insert or update a cache entry.
    pub async fn upsert(&self, channel_id: &str, data: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO subscription_cache (channel_id, data, last_updated)
            VALUES (?, ?, ?)
            ON CONFLICT(channel_id) DO UPDATE SET
                data = excluded.data,
                last_updated = excluded.last_updated
            "#,
        )
        .bind(channel_id)
        .bind(data)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Delete a cache entry by channel ID.
    pub async fn delete(&self, channel_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM subscription_cache WHERE channel_id = ?")
            .bind(channel_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get all cache entries.
    pub async fn get_all(&self) -> Result<Vec<SubscriptionCacheEntry>, sqlx::Error> {
        sqlx::query_as::<_, SubscriptionCacheEntry>(
            "SELECT channel_id, data, last_updated FROM subscription_cache",
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Get channel IDs with cache entries older than the specified minutes.
    pub async fn get_stale(&self, older_than_minutes: i64) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar::<_, String>(
            "SELECT channel_id FROM subscription_cache WHERE last_updated < datetime('now', ?)",
        )
        .bind(format!("-{} minutes", older_than_minutes))
        .fetch_all(&self.pool)
        .await
    }

    /// Update cached videos for a channel.
    pub async fn update_videos(
        &self,
        channel_id: &str,
        videos: &serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        let data = serde_json::to_string(videos)
            .map_err(|e| sqlx::Error::Protocol(format!("JSON serialization failed: {}", e).into()))?;
        self.upsert(channel_id, &data).await
    }

    /// Update cached live streams for a channel.
    pub async fn update_live_streams(
        &self,
        channel_id: &str,
        videos: &serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        let data = serde_json::to_string(videos)
            .map_err(|e| sqlx::Error::Protocol(format!("JSON serialization failed: {}", e).into()))?;
        self.upsert(channel_id, &data).await
    }

    /// Update cached shorts for a channel.
    pub async fn update_shorts(
        &self,
        channel_id: &str,
        videos: &serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        let data = serde_json::to_string(videos)
            .map_err(|e| sqlx::Error::Protocol(format!("JSON serialization failed: {}", e).into()))?;
        self.upsert(channel_id, &data).await
    }

    /// Update cached community posts for a channel.
    pub async fn update_community_posts(
        &self,
        channel_id: &str,
        posts: &serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        let data = serde_json::to_string(posts)
            .map_err(|e| sqlx::Error::Protocol(format!("JSON serialization failed: {}", e).into()))?;
        self.upsert(channel_id, &data).await
    }

    /// Delete cache entries for multiple channels.
    pub async fn delete_channels(&self, channel_ids: &[String]) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        for channel_id in channel_ids {
            sqlx::query("DELETE FROM subscription_cache WHERE channel_id = ?")
                .bind(channel_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Get all channel IDs that have cached data.
    pub async fn get_all_channels(&self) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar::<_, String>("SELECT channel_id FROM subscription_cache")
            .fetch_all(&self.pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_cache_repo_new() {
        fn _assert_new(pool: SqlitePool) -> SubscriptionCacheRepository {
            SubscriptionCacheRepository::new(pool)
        }
    }

    #[tokio::test]
    async fn test_update_videos_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE subscription_cache (
                channel_id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SubscriptionCacheRepository::new(pool.clone());

        // Update videos
        let videos = serde_json::json!([
            {"id": "vid1", "title": "Video 1"},
            {"id": "vid2", "title": "Video 2"}
        ]);
        repo.update_videos("channel1", &videos).await.unwrap();

        // Verify
        let entry = repo.find_one("channel1").await.unwrap().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&entry.data).unwrap();
        assert_eq!(parsed[0]["id"], "vid1");
        assert_eq!(parsed[1]["title"], "Video 2");
    }

    #[tokio::test]
    async fn test_update_live_streams_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE subscription_cache (
                channel_id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SubscriptionCacheRepository::new(pool.clone());

        let live = serde_json::json!([
            {"id": "live1", "title": "Live Stream", "viewers": 1000}
        ]);
        repo.update_live_streams("channel1", &live).await.unwrap();

        let entry = repo.find_one("channel1").await.unwrap().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&entry.data).unwrap();
        assert_eq!(parsed[0]["id"], "live1");
    }

    #[tokio::test]
    async fn test_update_shorts_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE subscription_cache (
                channel_id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SubscriptionCacheRepository::new(pool.clone());

        let shorts = serde_json::json!([{"id": "short1", "title": "Short Video"}]);
        repo.update_shorts("channel1", &shorts).await.unwrap();

        let entry = repo.find_one("channel1").await.unwrap().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&entry.data).unwrap();
        assert_eq!(parsed[0]["id"], "short1");
    }

    #[tokio::test]
    async fn test_update_community_posts_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE subscription_cache (
                channel_id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SubscriptionCacheRepository::new(pool.clone());

        let posts = serde_json::json!([
            {"id": "post1", "content": "Hello world", "likes": 42}
        ]);
        repo.update_community_posts("channel1", &posts).await.unwrap();

        let entry = repo.find_one("channel1").await.unwrap().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&entry.data).unwrap();
        assert_eq!(parsed[0]["content"], "Hello world");
    }

    #[tokio::test]
    async fn test_delete_channels_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE subscription_cache (
                channel_id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SubscriptionCacheRepository::new(pool.clone());

        // Insert entries for multiple channels
        repo.upsert("ch1", "data1").await.unwrap();
        repo.upsert("ch2", "data2").await.unwrap();
        repo.upsert("ch3", "data3").await.unwrap();

        // Delete some channels
        repo.delete_channels(&[String::from("ch1"), String::from("ch3")])
            .await
            .unwrap();

        // Verify ch1 and ch3 are gone, ch2 remains
        assert!(repo.find_one("ch1").await.unwrap().is_none());
        assert!(repo.find_one("ch2").await.unwrap().is_some());
        assert!(repo.find_one("ch3").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_all_channels_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE subscription_cache (
                channel_id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SubscriptionCacheRepository::new(pool.clone());

        // Initially empty
        let channels = repo.get_all_channels().await.unwrap();
        assert!(channels.is_empty());

        // Insert entries
        repo.upsert("ch1", "data1").await.unwrap();
        repo.upsert("ch2", "data2").await.unwrap();

        // Get all channels
        let channels = repo.get_all_channels().await.unwrap();
        assert_eq!(channels.len(), 2);
        assert!(channels.contains(&"ch1".to_string()));
        assert!(channels.contains(&"ch2".to_string()));
    }
}
