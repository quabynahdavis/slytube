// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            EnvFilter::new("info,opentubex=debug,slytube_lib=debug")
        } else {
            EnvFilter::new("warn,opentubex=info,slytube_lib=info")
        }
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_ansi(cfg!(debug_assertions)),
        )
        .init();
}

fn main() {
    init_tracing();
    tracing::info!("Starting Slytube");
    slytube_lib::run()
}
