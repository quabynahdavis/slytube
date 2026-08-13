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

    /// Bulk add videos (used by sync import).
    pub async fn add_videos_bulk(
        &self,
        playlist_id: &str,
        videos: &[PlaylistVideo],
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        for video in videos {
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO playlist_videos (playlist_id, video_id, position)
                VALUES (?, ?, ?)
                "#,
            )
            .bind(playlist_id)
            .bind(&video.video_id)
            .bind(video.position)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Bulk remove videos by ID.
    pub async fn remove_videos_bulk(
        &self,
        playlist_id: &str,
        video_ids: &[String],
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        for video_id in video_ids {
            sqlx::query("DELETE FROM playlist_videos WHERE playlist_id = ? AND video_id = ?")
                .bind(playlist_id)
                .bind(video_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Get all playlist videos ordered by position.
    pub async fn get_videos_ordered(
        &self,
        playlist_id: &str,
    ) -> Result<Vec<PlaylistVideo>, sqlx::Error> {
        sqlx::query_as::<_, PlaylistVideo>(
            "SELECT playlist_id, video_id, position FROM playlist_videos WHERE playlist_id = ? ORDER BY position ASC",
        )
        .bind(playlist_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Check if a video exists in a playlist.
    pub async fn has_video(
        &self,
        playlist_id: &str,
        video_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM playlist_videos WHERE playlist_id = ? AND video_id = ?",
        )
        .bind(playlist_id)
        .bind(video_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
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

    #[tokio::test]
    async fn test_add_videos_bulk_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE playlists (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE playlist_videos (
                playlist_id TEXT NOT NULL,
                video_id TEXT NOT NULL,
                position INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (playlist_id, video_id)
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = PlaylistsRepository::new(pool.clone());

        // Create a playlist
        sqlx::query("INSERT INTO playlists (id, profile_id, name, description, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind("pl1")
            .bind("profile1")
            .bind("My Playlist")
            .bind("Test playlist")
            .bind("2024-01-01T00:00:00Z")
            .execute(&pool)
            .await
            .unwrap();

        // Bulk add videos
        let videos = vec![
            PlaylistVideo {
                playlist_id: "pl1".to_string(),
                video_id: "vid1".to_string(),
                position: 0,
            },
            PlaylistVideo {
                playlist_id: "pl1".to_string(),
                video_id: "vid2".to_string(),
                position: 1,
            },
            PlaylistVideo {
                playlist_id: "pl1".to_string(),
                video_id: "vid3".to_string(),
                position: 2,
            },
        ];
        repo.add_videos_bulk("pl1", &videos).await.unwrap();

        // Verify videos were added
        let stored = repo.get_videos_ordered("pl1").await.unwrap();
        assert_eq!(stored.len(), 3);
        assert_eq!(stored[0].video_id, "vid1");
        assert_eq!(stored[1].video_id, "vid2");
        assert_eq!(stored[2].video_id, "vid3");
    }

    #[tokio::test]
    async fn test_remove_videos_bulk_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE playlists (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE playlist_videos (
                playlist_id TEXT NOT NULL,
                video_id TEXT NOT NULL,
                position INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (playlist_id, video_id)
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = PlaylistsRepository::new(pool.clone());

        sqlx::query("INSERT INTO playlists (id, profile_id, name, description, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind("pl1")
            .bind("profile1")
            .bind("My Playlist")
            .bind("Test playlist")
            .bind("2024-01-01T00:00:00Z")
            .execute(&pool)
            .await
            .unwrap();

        // Add videos
        let videos = vec![
            PlaylistVideo {
                playlist_id: "pl1".to_string(),
                video_id: "vid1".to_string(),
                position: 0,
            },
            PlaylistVideo {
                playlist_id: "pl1".to_string(),
                video_id: "vid2".to_string(),
                position: 1,
            },
            PlaylistVideo {
                playlist_id: "pl1".to_string(),
                video_id: "vid3".to_string(),
                position: 2,
            },
        ];
        repo.add_videos_bulk("pl1", &videos).await.unwrap();

        // Bulk remove some videos
        repo.remove_videos_bulk("pl1", &[String::from("vid1"), String::from("vid3")])
            .await
            .unwrap();

        // Verify only vid2 remains
        let stored = repo.get_videos_ordered("pl1").await.unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].video_id, "vid2");
    }

    #[tokio::test]
    async fn test_has_video_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE playlists (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE playlist_videos (
                playlist_id TEXT NOT NULL,
                video_id TEXT NOT NULL,
                position INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (playlist_id, video_id)
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = PlaylistsRepository::new(pool.clone());

        sqlx::query("INSERT INTO playlists (id, profile_id, name, description, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind("pl1")
            .bind("profile1")
            .bind("My Playlist")
            .bind("Test playlist")
            .bind("2024-01-01T00:00:00Z")
            .execute(&pool)
            .await
            .unwrap();

        // Add a video
        repo.add_video("pl1", "vid1", 0).await.unwrap();

        // Check has_video
        assert!(repo.has_video("pl1", "vid1").await.unwrap());
        assert!(!repo.has_video("pl1", "vid2").await.unwrap());
        assert!(!repo.has_video("pl_nonexistent", "vid1").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_videos_ordered_with_db() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE playlists (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE playlist_videos (
                playlist_id TEXT NOT NULL,
                video_id TEXT NOT NULL,
                position INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (playlist_id, video_id)
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = PlaylistsRepository::new(pool.clone());

        sqlx::query("INSERT INTO playlists (id, profile_id, name, description, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind("pl1")
            .bind("profile1")
            .bind("My Playlist")
            .bind("Test playlist")
            .bind("2024-01-01T00:00:00Z")
            .execute(&pool)
            .await
            .unwrap();

        // Add videos in non-sequential position order
        let videos = vec![
            PlaylistVideo {
                playlist_id: "pl1".to_string(),
                video_id: "vid3".to_string(),
                position: 2,
            },
            PlaylistVideo {
                playlist_id: "pl1".to_string(),
                video_id: "vid1".to_string(),
                position: 0,
            },
            PlaylistVideo {
                playlist_id: "pl1".to_string(),
                video_id: "vid2".to_string(),
                position: 1,
            },
        ];
        repo.add_videos_bulk("pl1", &videos).await.unwrap();

        // Verify videos are ordered by position
        let ordered = repo.get_videos_ordered("pl1").await.unwrap();
        assert_eq!(ordered.len(), 3);
        assert_eq!(ordered[0].video_id, "vid1");
        assert_eq!(ordered[0].position, 0);
        assert_eq!(ordered[1].video_id, "vid2");
        assert_eq!(ordered[1].position, 1);
        assert_eq!(ordered[2].video_id, "vid3");
        assert_eq!(ordered[2].position, 2);
    }
}
