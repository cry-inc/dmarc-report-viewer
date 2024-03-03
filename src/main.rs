#![forbid(unsafe_code)]

mod config;
mod dmarc;
mod http;
mod imap;
mod state;

use crate::dmarc::extract_reports;
use crate::http::run_http_server;
use crate::imap::get_mails;
use crate::state::AppState;
use anyhow::{Context, Result};
use config::Configuration;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Create config from args and ENV variables.
    // Will exit early in case of error or help and version command.
    let config = Configuration::new();

    // Set up basic logging to stdout
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_ansi(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up default tracing subscriber");

    info!("DMARC Report Analyzer");

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

fn start_bg_task(
    config: Configuration,
    state: Arc<Mutex<AppState>>,
    mut stop_signal: Receiver<()>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        info!(
            "Started background task with check interval of {} secs",
            config.imap_check_interval
        );
        loop {
            match bg_update(&config, &state).await {
                Ok(..) => info!("Finished update cycle without errors"),
                Err(err) => error!("Failed updated cycle: {err:#}"),
            };
            let duration = Duration::from_secs(config.imap_check_interval);
            tokio::select! {
                _ = tokio::time::sleep(duration) => {},
                _ = stop_signal.recv() => { break; },
            }
        }
    })
}

async fn bg_update(config: &Configuration, state: &Arc<Mutex<AppState>>) -> Result<()> {
    info!("Downloading mails...");
    let mails = get_mails(&config).context("Failed to get mails")?;
    state.lock().expect("Failed to lock app state").mails = mails.len();
    info!("Downloaded {} mails", mails.len());
    info!("Parsing mails...");
    for mail in mails {
        let reports = extract_reports(&mail).context("Failed to extract reports")?;
        for report in reports {
            info!("Report: {report:#?}");
        }
    }
    info!("Finished parsing all mails");
    Ok(())
}
