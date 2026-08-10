#[allow(dead_code)]
mod db;
mod potoken;
mod yt_dlp;

use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            yt_dlp::yt_dlp_get_info,
            yt_dlp::yt_dlp_get_playback_info,
            yt_dlp::yt_dlp_download,
            yt_dlp::yt_dlp_cancel,
            yt_dlp::yt_dlp_list,
            potoken::generate_po_token,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
