#![forbid(unsafe_code)]

mod background;
mod config;
mod http;
mod imap;
mod mail;
mod parser;
mod report;
mod state;
mod summary;
mod xml_error;
mod xml_file;

use crate::background::start_bg_task;
use crate::http::run_http_server;
use crate::state::AppState;
use anyhow::{Context, Result};
use config::Configuration;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::channel;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Create config from args and ENV variables.
    // Will exit early in case of error or help and version command.
    let config = Configuration::new();

    // Set up basic logging to stdout
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(config.log_level)
        .with_target(false)
        .with_ansi(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up default tracing subscriber");

    let version = env!("CARGO_PKG_VERSION");
    info!("DMARC Report Analyzer {version}");
    info!("Log Level: {}", config.log_level);

    // Prepare shared application state
    let state = Arc::new(Mutex::new(AppState::default()));

    // Start background task
    let (stop_sender, stop_receiver) = channel(1);
    let bg_handle = start_bg_task(config.clone(), state.clone(), stop_receiver);

    // Starting HTTP server
    run_http_server(&config, state.clone())
        .await
        .context("Failed to start HTTP server")?;

    // Shutdown rest of app after HTTP server stopped
    info!("HTTP server stopped");
    info!("Shutting down background task...");
    stop_sender
        .send(())
        .await
        .expect("Failed to send background task shutdown signal");
    bg_handle.await.expect("Failed to join background task");
    info!("Background task stopped, application shutdown completed!");

    Ok(())
}
