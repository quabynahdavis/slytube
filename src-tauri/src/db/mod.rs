use std::path::PathBuf;

use directories::ProjectDirs;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use tauri::{AppHandle, Manager};

pub mod models;
pub mod settings;

/// Database pool wrapper for managing SQLite connections.
#[derive(Debug, Clone)]
pub struct DbPool {
    pool: SqlitePool,
}

impl DbPool {
    /// Get a reference to the underlying sqlx pool.
    pub fn inner(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get the pool as a mutable reference.
    pub fn inner_mut(&mut self) -> &mut SqlitePool {
        &mut self.pool
    }
}

impl std::ops::Deref for DbPool {
    type Target = SqlitePool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

/// Initialize the database: create the data directory, establish the pool, and run migrations.
pub async fn init_db(app_handle: &AppHandle) -> Result<DbPool, sqlx::Error> {
    let data_dir = get_data_dir(app_handle);
    std::fs::create_dir_all(&data_dir)?;

    let db_path = data_dir.join("slytube.db");
    let db_url = format!("sqlite://{}", db_path.display());

    // Create the database file if it doesn't exist
    if !db_path.exists() {
        Sqlite::create_database(&db_url).await?;
    }

    // Configure the connection pool
    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(DbPool { pool })
}

/// Get the data directory for the application.
fn get_data_dir(app_handle: &AppHandle) -> PathBuf {
    // First try to use Tauri's path resolver
    if let Ok(path) = app_handle.path().app_data_dir() {
        return path;
    }

    // Fall back to directories crate
    if let Some(proj_dirs) = ProjectDirs::from("com", "slytube", "slytube") {
        return proj_dirs.data_dir().to_path_buf();
    }

    // Last resort: use the executable directory
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_pool_deref() {
        // Just verify the types compile correctly
        fn _assert_deref(pool: &DbPool) -> &SqlitePool {
            pool
        }
    }
}
