use sqlx::SqlitePool;

use crate::db::models::{Playlist, PlaylistVideo};

/// Repository for playlist CRUD operations.
#[derive(Debug, Clone)]
pub struct PlaylistsRepository {
    pool: SqlitePool,
}

impl PlaylistsRepository {
    /// Create a new PlaylistsRepository with the given pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find all playlists for a profile.
    pub async fn find_all(&self, profile_id: &str) -> Result<Vec<Playlist>, sqlx::Error> {
        sqlx::query_as::<_, Playlist>(
            "SELECT id, profile_id, name, description, created_at FROM playlists WHERE profile_id = ?",
        )
        .bind(profile_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Find a single playlist by its ID.
    pub async fn find_one(&self, id: &str) -> Result<Option<Playlist>, sqlx::Error> {
        sqlx::query_as::<_, Playlist>(
            "SELECT id, profile_id, name, description, created_at FROM playlists WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Create a new playlist.
    pub async fn create(&self, playlist: &Playlist) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO playlists (id, profile_id, name, description, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&playlist.id)
        .bind(&playlist.profile_id)
        .bind(&playlist.name)
        .bind(&playlist.description)
        .bind(&playlist.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Update an existing playlist.
    pub async fn update(&self, playlist: &Playlist) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE playlists
            SET name = ?, description = ?
            WHERE id = ?
            "#,
        )
        .bind(&playlist.name)
        .bind(&playlist.description)
        .bind(&playlist.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Delete a playlist by its ID.
    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM playlists WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get all videos in a playlist.
    pub async fn get_videos(&self, playlist_id: &str) -> Result<Vec<PlaylistVideo>, sqlx::Error> {
        sqlx::query_as::<_, PlaylistVideo>(
            "SELECT playlist_id, video_id, position FROM playlist_videos WHERE playlist_id = ? ORDER BY position",
        )
        .bind(playlist_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Add a video to a playlist.
    pub async fn add_video(
        &self,
        playlist_id: &str,
        video_id: &str,
        position: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO playlist_videos (playlist_id, video_id, position)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(playlist_id)
        .bind(video_id)
        .bind(position)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Remove a video from a playlist.
    pub async fn remove_video(
        &self,
        playlist_id: &str,
        video_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM playlist_videos WHERE playlist_id = ? AND video_id = ?")
            .bind(playlist_id)
            .bind(video_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Reorder videos in a playlist.
    pub async fn reorder_videos(
        &self,
        playlist_id: &str,
        video_ids: &[String],
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        for (idx, video_id) in video_ids.iter().enumerate() {
            sqlx::query("UPDATE playlist_videos SET position = ? WHERE playlist_id = ? AND video_id = ?")
                .bind(idx as i64)
                .bind(playlist_id)
                .bind(video_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playlists_repo_new() {
        fn _assert_new(pool: SqlitePool) -> PlaylistsRepository {
            PlaylistsRepository::new(pool)
        }
    }
}
