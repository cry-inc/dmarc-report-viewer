mod config;
mod dmarc;
mod http;
mod imap;

use crate::dmarc::extract_reports;
use crate::http::run_http_server;
use crate::imap::get_mails;
use anyhow::{Context, Result};
use clap::Parser;
use config::Configuration;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Configuration::parse();

    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_ansi(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up default tracing subscriber");

    info!("DMARC Report Analyzer");

    info!("Downloading mails...");
    let mails = get_mails(&config).context("Failed to get mails")?;
    info!("Downloaded {} mails", mails.len());

    info!("Parsing mails...");
    for mail in mails {
        let reports = extract_reports(&mail).context("Failed to extract reports")?;
        for report in reports {
            info!("Report: {report:#?}");
        }
    }
    info!("Finished parsing all mails");

    run_http_server(&config)
        .await
        .context("Failed to start HTTP server")?;

    info!("Shutting down...");
    Ok(())
}
