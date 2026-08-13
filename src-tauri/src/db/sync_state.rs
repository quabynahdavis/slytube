use sqlx::SqlitePool;

/// Repository for sync state CRUD operations.
#[derive(Debug, Clone)]
pub struct SyncStateRepository {
    pool: SqlitePool,
}

impl SyncStateRepository {
    /// Create a new SyncStateRepository with the given pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get a sync state value by key.
    pub async fn get_state(&self, key: &str) -> Result<Option<String>, sqlx::Error> {
        let result: Option<(String,)> = sqlx::query_as("SELECT value FROM sync_state WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result.map(|r| r.0))
    }

    /// Set a sync state value (insert or update).
    pub async fn set_state(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO sync_state (key, value) VALUES (?, ?)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Delete a sync state entry by key.
    pub async fn delete_state(&self, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM sync_state WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_state_repo_new() {
        fn _assert_new(pool: SqlitePool) -> SyncStateRepository {
            SyncStateRepository::new(pool)
        }
    }

    #[tokio::test]
    async fn test_set_and_get_state_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE sync_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SyncStateRepository::new(pool.clone());

        // Initially None
        assert!(repo.get_state("last_sync").await.unwrap().is_none());

        // Set state
        repo.set_state("last_sync", "1700000000").await.unwrap();

        // Get state
        let value = repo.get_state("last_sync").await.unwrap();
        assert_eq!(value, Some("1700000000".to_string()));

        // Update state
        repo.set_state("last_sync", "1700000100").await.unwrap();
        let value = repo.get_state("last_sync").await.unwrap();
        assert_eq!(value, Some("1700000100".to_string()));
    }

    #[tokio::test]
    async fn test_delete_state_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE sync_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SyncStateRepository::new(pool.clone());

        // Set state
        repo.set_state("key1", "value1").await.unwrap();
        repo.set_state("key2", "value2").await.unwrap();

        // Delete one
        repo.delete_state("key1").await.unwrap();

        // Verify key1 is gone, key2 remains
        assert!(repo.get_state("key1").await.unwrap().is_none());
        assert_eq!(
            repo.get_state("key2").await.unwrap(),
            Some("value2".to_string())
        );
    }

    #[tokio::test]
    async fn test_get_state_missing_key() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE sync_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SyncStateRepository::new(pool.clone());

        // Non-existent key returns None
        let result = repo.get_state("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_set_state_overwrite_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE sync_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SyncStateRepository::new(pool.clone());

        // Set multiple times (should upsert)
        repo.set_state("counter", "1").await.unwrap();
        repo.set_state("counter", "2").await.unwrap();
        repo.set_state("counter", "3").await.unwrap();

        let value = repo.get_state("counter").await.unwrap();
        assert_eq!(value, Some("3".to_string()));
    }
}
