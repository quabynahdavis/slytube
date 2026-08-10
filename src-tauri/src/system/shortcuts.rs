use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

/// Register global shortcuts for the application.
///
/// - Play/Pause: Media key or custom
/// - Next: Media key or custom
/// - Previous: Media key or custom
/// - Toggle favorite: Ctrl/Cmd + D
pub fn register_shortcuts(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Toggle favorite: Ctrl/Cmd + D
    let app_handle = app.clone();
    app.global_shortcut()
        .on_shortcut("CmdOrCtrl+D", move |_, _, _| {
            let _ = app_handle.emit("shortcut-toggle-favorite", ());
        })?;

    // Play/Pause media key
    let app_handle = app.clone();
    app.global_shortcut()
        .on_shortcut("MediaPlayPause", move |_, _, _| {
            let _ = app_handle.emit("shortcut-media-play-pause", ());
        })?;

    // Next track media key
    let app_handle = app.clone();
    app.global_shortcut()
        .on_shortcut("MediaNextTrack", move |_, _, _| {
            let _ = app_handle.emit("shortcut-media-next", ());
        })?;

    // Previous track media key
    let app_handle = app.clone();
    app.global_shortcut()
        .on_shortcut("MediaPreviousTrack", move |_, _, _| {
            let _ = app_handle.emit("shortcut-media-previous", ());
        })?;

    Ok(())
}
