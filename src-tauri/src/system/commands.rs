use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};

/// Show the main window.
#[tauri::command]
pub fn system_show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Hide the main window.
#[tauri::command]
pub fn system_hide_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Toggle the main window visibility.
#[tauri::command]
pub fn system_toggle_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Get the application version.
#[tauri::command]
pub fn system_get_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

/// Check for application updates.
#[tauri::command]
pub async fn system_check_for_updates(app: AppHandle) -> Result<String, String> {
    // Placeholder for update checking logic
    // In production, this would query an update server or GitHub releases
    let current_version = app.package_info().version.to_string();
    tracing::info!("Checking for updates (current version: {})", current_version);
    Ok(format!("You are running version {}", current_version))
}

/// Open a URL in the system's default browser.
#[tauri::command]
pub fn system_open_external(_app: AppHandle, url: String) -> Result<(), String> {
    tauri_plugin_opener::open_url(&url, None::<&str>).map_err(|e| e.to_string())?;
    Ok(())
}

/// Center the window on the screen.
#[tauri::command]
pub fn system_center_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(monitor) = window.current_monitor() {
            if let Some(monitor) = monitor {
                let monitor_size = monitor.size();
                let window_size = window.outer_size().map_err(|e| e.to_string())?;

                let x = (monitor_size.width - window_size.width) / 2;
                let y = (monitor_size.height - window_size.height) / 2;

                window
                    .set_position(PhysicalPosition::new(x, y))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

/// Set the window to fullscreen mode.
#[tauri::command]
pub fn system_set_fullscreen(app: AppHandle, fullscreen: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_fullscreen(fullscreen).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Get the window size.
#[tauri::command]
pub fn system_get_window_size(app: AppHandle) -> Result<PhysicalSize<u32>, String> {
    if let Some(window) = app.get_webview_window("main") {
        window.outer_size().map_err(|e| e.to_string())
    } else {
        Err("Main window not found".to_string())
    }
}
