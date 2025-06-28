#![forbid(unsafe_code)]

mod background;
mod cache_map;
mod config;
mod dmarc;
mod geolocate;
mod hasher;
mod http;
mod imap;
mod mail;
mod state;
mod tls;
mod unpack;
mod whois;

use crate::background::start_bg_task;
use crate::http::run_http_server;
use crate::state::AppState;
use anyhow::{Context, Result};
use config::Configuration;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc::channel};
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

    // Log app name and version
    let version = env!("CARGO_PKG_VERSION");
    info!("DMARC Report Analyzer {version}");

    // Inject git hash for logging during Github builds.
    // Other builds, like normal local dev builds do not support this.
    let git_hash = option_env!("GITHUB_SHA").unwrap_or("n/a");
    let git_ref = option_env!("GITHUB_REF_NAME").unwrap_or("n/a");
    info!("Git-Hash: {git_hash} ({git_ref})");

    // Make configuration visible in logs
    config.log();

    // Prepare shared application state
    let state = Arc::new(Mutex::new(AppState::new()));

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
