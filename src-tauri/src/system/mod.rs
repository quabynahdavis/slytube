pub mod commands;
pub mod protocol;
pub mod shortcuts;
pub mod tray;

use tauri::AppHandle;

/// Initialize the system module (tray, shortcuts, protocol handling).
pub fn init(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Create system tray
    tray::create_tray(app)?;
    tracing::info!("System tray initialized");

    // Register global shortcuts
    shortcuts::register_shortcuts(app)?;
    tracing::info!("Global shortcuts registered");

    Ok(())
}

/// Handle deep link URLs (opentubex://).
pub fn handle_deep_link(app: &AppHandle, url: String) {
    protocol::handle_deep_link(app, url);
}
