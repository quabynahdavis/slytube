#[allow(dead_code)]
mod commands;
#[allow(dead_code)]
mod db;
#[allow(dead_code)]
mod http_client;
mod potoken;
mod system;
mod yt_dlp;

use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::http_client::HttpClient;

// Learn more about Tauri commands at https://tauri.app/develop/calling-calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Handle deep link URLs (opentubex://).
#[tauri::command]
fn system_deep_link(url: String, app: AppHandle) {
    system::handle_deep_link(&app, url);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_http::init());

    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_global_shortcut::Builder::new().build());

    builder
        .setup(|app| {
            // Initialize HTTP client for YouTube/Invidious API calls
            let http_client = Arc::new(HttpClient::new()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?);
            app.manage(http_client);
            tracing::info!("HTTP client initialized");

            // Initialize yt-dlp state for managing active downloads
            app.manage(yt_dlp::YtDlpState::new());
            tracing::info!("YtDlpState initialized");

            // Initialize PoToken state for token generation tracking
            app.manage(potoken::PoTokenState::new());
            tracing::info!("PoTokenState initialized");

            // Initialize the database pool and manage it as state
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                match db::init_db(&app_handle).await {
                    Ok(pool) => {
                        app_handle.manage(pool);
                        tracing::info!("Database initialized successfully");
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize database: {}", e);
                        return Err(Box::new(e) as Box<dyn std::error::Error>);
                    }
                }
                Ok(())
            })?;

            // Initialize system module (tray, shortcuts)
            if let Err(e) = system::init(app.handle()) {
                tracing::error!("Failed to initialize system module: {}", e);
            }

            Ok(())
        })
        .on_window_event(|_window, _event| {
            // Handle window events if needed
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            system_deep_link,
            // YouTube InnerTube API
            commands::youtube::get_video_info,
            commands::youtube::search_videos,
            commands::youtube::get_trending,
            commands::youtube::get_channel_info,
            commands::youtube::get_channel_videos,
            commands::youtube::get_comments,
            // Invidious API
            commands::invidious::invidious_get_video,
            commands::invidious::invidious_search,
            commands::invidious::invidious_get_trending,
            commands::invidious::invidious_get_channel,
            commands::invidious::invidious_get_playlist,
            commands::invidious::invidious_get_comments,
            commands::invidious::invidious_get_instances,
            commands::invidious::invidious_test_instance,
            // yt-dlp
            yt_dlp::yt_dlp_get_info,
            yt_dlp::yt_dlp_get_playback_info,
            yt_dlp::yt_dlp_download,
            yt_dlp::yt_dlp_cancel,
            yt_dlp::yt_dlp_list,
            potoken::generate_po_token,
            // System commands
            system::commands::system_show_main_window,
            system::commands::system_hide_main_window,
            system::commands::system_toggle_window,
            system::commands::system_get_version,
            system::commands::system_check_for_updates,
            system::commands::system_open_external,
            system::commands::system_center_window,
            system::commands::system_set_fullscreen,
            system::commands::system_get_window_size,
            // Settings
            db::commands::db_settings_find_all,
            db::commands::db_settings_find_one,
            db::commands::db_settings_upsert,
            // Profiles
            db::commands::db_profiles_find_all,
            db::commands::db_profiles_find_one,
            db::commands::db_profiles_create,
            db::commands::db_profiles_update,
            db::commands::db_profiles_delete,
            // Profile subscriptions
            db::commands::db_profiles_get_subscriptions,
            db::commands::db_profiles_add_subscription,
            db::commands::db_profiles_remove_subscription,
            // Playlists
            db::commands::db_playlists_find_all,
            db::commands::db_playlists_find_one,
            db::commands::db_playlists_create,
            db::commands::db_playlists_update,
            db::commands::db_playlists_delete,
            // Playlist videos
            db::commands::db_playlists_get_videos,
            db::commands::db_playlists_add_video,
            db::commands::db_playlists_remove_video,
            // History
            db::commands::db_history_find_all,
            db::commands::db_history_find_one,
            db::commands::db_history_upsert,
            db::commands::db_history_delete,
            db::commands::db_history_clear,
            // Watch stats
            db::commands::db_watch_stats_add,
            db::commands::db_watch_stats_get_total,
            // Search history
            db::commands::db_search_history_find_all,
            db::commands::db_search_history_add,
            db::commands::db_search_history_clear,
            // Subscription cache
            db::commands::db_subscription_cache_find_one,
            db::commands::db_subscription_cache_upsert,
            db::commands::db_subscription_cache_get_all,
            // Tab sessions
            db::commands::db_tab_sessions_save,
            db::commands::db_tab_sessions_get_latest,
            db::commands::db_tab_sessions_clear,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
