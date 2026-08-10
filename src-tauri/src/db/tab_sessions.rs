use sqlx::SqlitePool;

use crate::db::models::TabSession;

/// Repository for tab session persistence.
#[derive(Debug, Clone)]
pub struct TabSessionsRepository {
    pool: SqlitePool,
}

impl TabSessionsRepository {
    /// Create a new TabSessionsRepository with the given pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Save a tab session and return its ID.
    pub async fn save(&self, data: &str) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("INSERT INTO tab_sessions (data) VALUES (?)")
            .bind(data)
            .execute(&self.pool)
            .await?;
        Ok(result.last_insert_rowid())
    }

    /// Get the most recent tab session.
    pub async fn get_latest(&self) -> Result<Option<TabSession>, sqlx::Error> {
        sqlx::query_as::<_, TabSession>(
            "SELECT id, data, created_at FROM tab_sessions ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
    }

    /// Clear all tab sessions.
    pub async fn clear_all(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM tab_sessions")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_sessions_repo_new() {
        fn _assert_new(pool: SqlitePool) -> TabSessionsRepository {
            TabSessionsRepository::new(pool)
        }
    }
}
