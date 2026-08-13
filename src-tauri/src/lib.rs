#[allow(dead_code)]
mod commands;
#[allow(dead_code)]
mod community;
#[allow(dead_code)]
mod db;
#[allow(dead_code)]
mod http_client;
mod extractor;
mod image_cache;
mod potoken;
mod sync;
mod system;
mod yt_dlp;

use std::sync::Arc;

use tauri::{AppHandle, Manager, State};

use crate::http_client::HttpClient;

/// Handle deep link URLs (opentubex://).
#[tauri::command]
fn system_deep_link(url: String, app: AppHandle) {
    system::handle_deep_link(&app, url);
}

/// Fetch a YouTube image through the Rust-side cache and return it as a
/// base64 data URL. Avoids direct webview connections to YouTube CDNs.
#[tauri::command]
async fn image_cache_get(
    url: String,
    cache: State<'_, image_cache::ImageCache>,
    http: State<'_, crate::http_client::SharedHttpClient>,
) -> Result<String, String> {
    cache
        .get_image(&url, &http)
        .await
        .map_err(|e| e.to_string())
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
        .register_uri_scheme_protocol("imgcache", move |ctx, request| {
            image_cache::handle_protocol_request(ctx.app_handle(), &request)
        })
        .setup(|app| {
            // Initialize HTTP client for YouTube/Invidious API calls
            let http_client = Arc::new(HttpClient::new()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?);
            app.manage(http_client);
            tracing::info!("HTTP client initialized");

            // Initialize image cache for YouTube thumbnails (avoids direct
            // webview connections to YouTube CDNs for privacy / CORS reasons).
            let image_cache = image_cache::ImageCache::new();
            image_cache.clone().start_cleanup_task();
            app.manage(image_cache);
            tracing::info!("Image cache initialized");

            // Initialize yt-dlp state for managing active downloads
            app.manage(yt_dlp::YtDlpState::new());
            tracing::info!("YtDlpState initialized");

            // Initialize PoToken state for token generation tracking
            app.manage(potoken::PoTokenState::new());
            tracing::info!("PoTokenState initialized");

            // Initialize sync manager for tracking sync operations
            app.manage(sync::commands::SyncManager::new());
            tracing::info!("SyncManager initialized");

            // Initialize extractor pending-request correlation state
            app.manage(extractor::PendingExtractions::new());
            tracing::info!("PendingExtractions initialized");

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

            // Create the hidden extractor webview that runs youtubei.js.
            // This webview is never shown to the user — it exists solely to
            // execute youtubei.js (Innertube) for extraction and BotGuard VM
            // for PoToken generation. It loads dist/extractor.html which is
            // built from src/extractor/index.html as a separate Rollup entry.
            #[cfg(desktop)]
            {
                use tauri::{WebviewUrl, WebviewWindowBuilder};

                let extractor_url = WebviewUrl::App("extractor.html".into());
                match WebviewWindowBuilder::new(app, "extractor", extractor_url)
                    .title("Slytube Extractor")
                    .visible(false)
                    .inner_size(1920.0, 1080.0)
                    .decorations(false)
                    .build()
                {
                    Ok(_) => tracing::info!("Extractor hidden webview created"),
                    Err(e) => tracing::warn!("Failed to create extractor webview: {} (may be created lazily on first use)", e),
                }
            }

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
            system_deep_link,
            image_cache_get,
            // Extractor (hidden webview youtubei.js bridge)
            extractor::extract,
            extractor::extraction_result,
            extractor::extractor_ready,
            // YouTube InnerTube API
            commands::youtube::get_video_info,
            commands::youtube::search_videos,
            commands::youtube::get_trending,
            commands::youtube::get_channel_info,
            commands::youtube::get_channel_videos,
            commands::youtube::get_comments,
            commands::youtube::get_search_suggestions,
            commands::youtube::get_playlist_info,
            commands::youtube::get_community_posts,
            commands::youtube::get_hashtag,
            // Invidious API
            commands::invidious::invidious_get_video,
            commands::invidious::invidious_search,
            commands::invidious::invidious_get_trending,
            commands::invidious::invidious_get_channel,
            commands::invidious::invidious_get_playlist,
            commands::invidious::invidious_get_comments,
            commands::invidious::invidious_get_instances,
            commands::invidious::invidious_test_instance,
            commands::invidious::invidious_get_dash_manifest,
            commands::invidious::invidious_get_dash_url,
            commands::invidious::invidious_get_format_streams,
            commands::invidious::invidious_get_popular,
            commands::invidious::invidious_get_channel_videos,
            commands::invidious::invidious_resolve_url,
            commands::invidious::invidious_get_channel_tabs,
            commands::invidious::invidious_get_channel_shorts,
            commands::invidious::invidious_get_channel_live,
            commands::invidious::invidious_get_channel_playlists,
            commands::invidious::invidious_get_channel_releases,
            commands::invidious::invidious_get_channel_podcasts,
            commands::invidious::invidious_get_channel_courses,
            commands::invidious::invidious_search_channel,
            commands::invidious::invidious_get_comment_replies,
            commands::invidious::invidious_get_search_suggestions,
            commands::invidious::invidious_search_with_filters,
            commands::invidious::invidious_get_community_posts,
            commands::invidious::invidious_get_community_post,
            commands::invidious::invidious_get_community_post_comments,
            commands::invidious::invidious_get_community_post_comment_replies,
            commands::invidious::invidious_get_hashtag,
            // Generic fetch command for frontend
            commands::invidious::invidious_fetch,
            commands::invidious::invidious_get_instances_list,
            // yt-dlp
            yt_dlp::yt_dlp_get_info,
            yt_dlp::yt_dlp_get_playback_info,
            yt_dlp::yt_dlp_download,
            yt_dlp::yt_dlp_cancel,
            yt_dlp::yt_dlp_list,
            yt_dlp::yt_dlp_check_binary,
            yt_dlp::yt_dlp_download_binary,
            potoken::generate_po_token,
            // Sync commands
            sync::commands::sync_test_connection,
            sync::commands::sync_register,
            sync::commands::sync_login,
            sync::commands::sync_delete_account,
            sync::commands::sync_prepare_key,
            sync::commands::sync_encrypt,
            sync::commands::sync_decrypt,
            sync::commands::sync_get_manifest,
            sync::commands::sync_get_collection,
            sync::commands::sync_upload_collection,
            sync::commands::sync_get_state,
            sync::commands::sync_start,
            sync::commands::sync_cancel,
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
            // Playlist bulk operations
            db::commands::db_playlists_add_videos_bulk,
            db::commands::db_playlists_remove_videos_bulk,
            // History
            db::commands::db_history_find_all,
            db::commands::db_history_find_one,
            db::commands::db_history_upsert,
            db::commands::db_history_delete,
            db::commands::db_history_clear,
            // History sync operations
            db::commands::db_history_apply_sync_changes,
            db::commands::db_history_get_newer_than,
            db::commands::db_history_update_progress,
            db::commands::db_history_delete_older_than,
            // Watch stats
            db::commands::db_watch_stats_add,
            db::commands::db_watch_stats_add_date,
            db::commands::db_watch_stats_get_total,
            // Search history
            db::commands::db_search_history_find_all,
            db::commands::db_search_history_add,
            db::commands::db_search_history_clear,
            // Subscription cache
            db::commands::db_subscription_cache_find_one,
            db::commands::db_subscription_cache_upsert,
            db::commands::db_subscription_cache_get_all,
            db::commands::db_subscription_cache_update,
            // Tab sessions
            db::commands::db_tab_sessions_save,
            db::commands::db_tab_sessions_get_latest,
            db::commands::db_tab_sessions_clear,
            // Sync state
            db::commands::db_sync_state_get,
            db::commands::db_sync_state_set,
            // Community integrations (SponsorBlock / DeArrow / RYD)
            community::sponsorblock_get_segments,
            community::sponsorblock_get_labels,
            community::sponsorblock_submit_segments,
            community::sponsorblock_vote,
            community::dearrow_get_data,
            community::dearrow_get_thumbnail,
            community::ryd_get_dislikes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
