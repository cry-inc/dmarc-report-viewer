mod config;

use anyhow::{Context, Result};
use clap::Parser;
use config::Configuration;
use dmarc_aggregate_parser::aggregate_report::feedback;
use imap::Client;
use rustls_connector::RustlsConnector;
use std::{io::Cursor, net::TcpStream};
use tracing::{info, warn};
use zip::ZipArchive;

fn get_mails(config: &Configuration) -> Result<Vec<Vec<u8>>> {
    let mut mails = Vec::new();
    let addr = (config.imap_host.as_str(), config.imap_port);
    let stream = TcpStream::connect(addr).context("Failed to connect to IMAP server")?;
    let connector =
        RustlsConnector::new_with_native_certs().expect("Failed to create Rust TLS connector");
    let tls_stream = connector
        .connect(&config.imap_host, stream)
        .context("Failed to set up TLS stream")?;
    let client = Client::new(tls_stream);
    let mut session = client
        .login(&config.imap_user, &config.imap_password)
        .expect("Failed to log in");
    let mailbox = session.select("INBOX").context("Failed to select inbox")?;
    if mailbox.exists > 0 {
        let sequence = format!("1:{}", mailbox.exists);
        let messages = session
            .fetch(sequence, "RFC822")
            .context("Failed to fetch first message")?;
        for message in messages.iter() {
            let body = message.body().context("Message did not have a body!")?;
            mails.push(body.to_vec());
        }
    }
    session.logout().context("Failed to log out")?;
    Ok(mails)
}

fn extract_reports(mail: &[u8]) -> Result<Vec<feedback>> {
    let parsed = mailparse::parse_mail(mail).context("Failed to parse mail body")?;
    let zip_bytes = parsed
        .get_body_raw()
        .context("Failed to get raw body of the message")?;
    let cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).context("Failed to open body as ZIP")?;
    let mut reports = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).context("Unable to get file from ZIP")?;
        if !file.name().ends_with(".xml") {
            warn!("{} is not an XML file, skipping...", file.name());
            continue;
        }
        let report = dmarc_aggregate_parser::parse_reader(&mut file)
            .context("Failed to parse XML as DMARC report")?;
        reports.push(report);
    }
    Ok(reports)
}

fn main() -> Result<()> {
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

    info!("Shutting down...");
    Ok(())
}
