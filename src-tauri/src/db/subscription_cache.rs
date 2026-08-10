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
}
