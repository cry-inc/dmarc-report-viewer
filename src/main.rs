#![forbid(unsafe_code)]

mod config;
mod dmarc_report;
mod http;
mod imap;
mod parser;
mod state;
mod summary;

use crate::http::run_http_server;
use crate::imap::get_mails;
use crate::parser::{extract_xml_files, parse_xml_file};
use crate::state::AppState;
use crate::summary::Summary;
use anyhow::{Context, Result};
use config::Configuration;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

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
    info!("Starting background update cycle");

    info!("Downloading mails...");
    let mails = get_mails(config).context("Failed to get mails")?;
    info!("Downloaded {} mails from IMAP inbox", mails.len());

    info!("Extracting XML files from mails...");
    let mut xml_files = Vec::new();
    for mail in &mails {
        match extract_xml_files(mail) {
            Ok(mut files) => xml_files.append(&mut files),
            Err(err) => warn!("Failed to extract XML files from mail: {err:#}"),
        }
    }
    info!("Extracted {} XML files from mails", xml_files.len());

    info!("Parsing XML files as DMARC reports...");
    let mut reports = Vec::new();
    for xml_file in &xml_files {
        match parse_xml_file(xml_file) {
            Ok(report) => reports.push(report),
            Err(err) => warn!("Failed to parse XML file as DMARC report: {err:#}"),
        }
    }
    info!("Parsed {} DMARC reports successfully", reports.len());

    info!("Creating report summary...");
    let summary = Summary::new(&mails, &xml_files, &reports);

    info!("Updating sharted state...");
    {
        let mut locked_state = state.lock().expect("Failed to lock app state");
        locked_state.mails = mails.len();
        locked_state.xml_files = xml_files.len();
        locked_state.summary = summary;
        locked_state.reports = reports;
    }
    info!("Finished updating shared state");

    Ok(())
}
