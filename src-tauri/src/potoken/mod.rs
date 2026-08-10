use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, WebviewWindowBuilder};

mod commands;
mod models;

pub use commands::*;
pub use models::*;

/// PoToken generation state.
pub struct PoTokenState {
    pub last_result: Mutex<Option<String>>,
    pub generation_count: Mutex<u64>,
}

impl PoTokenState {
    pub fn new() -> Self {
        Self {
            last_result: Mutex::new(None),
            generation_count: Mutex::new(0),
        }
    }
}

/// Get the botGuard script content bundled with the app.
pub fn get_botguard_script() -> &'static str {
    include_str!("../../binaries/botGuardScript.js")
}
