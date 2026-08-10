use chrono::Utc;
use sqlx::SqlitePool;

use crate::db::models::SearchEntry;

/// Repository for search history CRUD operations.
#[derive(Debug, Clone)]
pub struct SearchHistoryRepository {
    pool: SqlitePool,
}

impl SearchHistoryRepository {
    /// Create a new SearchHistoryRepository with the given pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find all search history entries, ordered by most recent.
    pub async fn find_all(&self, limit: i64) -> Result<Vec<SearchEntry>, sqlx::Error> {
        sqlx::query_as::<_, SearchEntry>(
            "SELECT id, query, timestamp FROM search_history ORDER BY timestamp DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Add a search query to history.
    pub async fn add(&self, query: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("INSERT INTO search_history (query, timestamp) VALUES (?, ?)")
            .bind(query)
            .bind(&now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Delete a search history entry by ID.
    pub async fn delete(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM search_history WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Clear all search history.
    pub async fn clear_all(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM search_history")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_history_repo_new() {
        fn _assert_new(pool: SqlitePool) -> SearchHistoryRepository {
            SearchHistoryRepository::new(pool)
        }
    }
}
