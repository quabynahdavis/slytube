use sqlx::SqlitePool;

use crate::db::models::Profile;

/// Repository for profile CRUD operations.
#[derive(Debug, Clone)]
pub struct ProfilesRepository {
    pool: SqlitePool,
}

impl ProfilesRepository {
    /// Create a new ProfilesRepository with the given pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find all profiles.
    pub async fn find_all(&self) -> Result<Vec<Profile>, sqlx::Error> {
        sqlx::query_as::<_, Profile>("SELECT id, name, avatar, created_at FROM profiles")
            .fetch_all(&self.pool)
            .await
    }

    /// Find a single profile by its ID.
    pub async fn find_one(&self, id: &str) -> Result<Option<Profile>, sqlx::Error> {
        sqlx::query_as::<_, Profile>(
            "SELECT id, name, avatar, created_at FROM profiles WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Create a new profile.
    pub async fn create(&self, profile: &Profile) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO profiles (id, name, avatar, created_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&profile.id)
        .bind(&profile.name)
        .bind(&profile.avatar)
        .bind(&profile.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Update an existing profile.
    pub async fn update(&self, profile: &Profile) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE profiles
            SET name = ?, avatar = ?
            WHERE id = ?
            "#,
        )
        .bind(&profile.name)
        .bind(&profile.avatar)
        .bind(&profile.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Delete a profile by its ID.
    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM profiles WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get all subscription channel IDs for a profile.
    pub async fn get_subscriptions(&self, profile_id: &str) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar::<_, String>(
            "SELECT channel_id FROM profile_subscriptions WHERE profile_id = ?",
        )
        .bind(profile_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Add a subscription to a profile.
    pub async fn add_subscription(
        &self,
        profile_id: &str,
        channel_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO profile_subscriptions (profile_id, channel_id)
            VALUES (?, ?)
            "#,
        )
        .bind(profile_id)
        .bind(channel_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Remove a subscription from a profile.
    pub async fn remove_subscription(
        &self,
        profile_id: &str,
        channel_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM profile_subscriptions WHERE profile_id = ? AND channel_id = ?",
        )
        .bind(profile_id)
        .bind(channel_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiles_repo_new() {
        fn _assert_new(pool: SqlitePool) -> ProfilesRepository {
            ProfilesRepository::new(pool)
        }
    }
}
