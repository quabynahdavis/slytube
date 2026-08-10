use chrono::Utc;
use sqlx::SqlitePool;

use crate::db::models::Setting;

/// Repository for settings CRUD operations.
#[derive(Debug, Clone)]
pub struct SettingsRepository {
    pool: SqlitePool,
}

impl SettingsRepository {
    /// Create a new SettingsRepository with the given pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find all settings.
    pub async fn find_all(&self) -> Result<Vec<Setting>, sqlx::Error> {
        sqlx::query_as::<_, Setting>("SELECT id, value, updated_at FROM settings")
            .fetch_all(&self.pool)
            .await
    }

    /// Find a single setting by its ID.
    pub async fn find_one(&self, id: &str) -> Result<Option<Setting>, sqlx::Error> {
        sqlx::query_as::<_, Setting>(
            "SELECT id, value, updated_at FROM settings WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Insert or update a setting.
    pub async fn upsert(&self, id: &str, value: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO settings (id, value, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(id)
        .bind(value)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Delete a setting by its ID.
    pub async fn delete(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM settings WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_repo_new() {
        // Just verify the types compile correctly
        fn _assert_new(pool: SqlitePool) -> SettingsRepository {
            SettingsRepository::new(pool)
        }
    }
}
