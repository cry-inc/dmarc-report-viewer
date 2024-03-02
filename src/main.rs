mod config;

use anyhow::{Context, Result};
use clap::Parser;
use config::Configuration;
use dmarc_aggregate_parser::aggregate_report::feedback;
use native_tls::TlsConnector;
use piz::ZipArchive;

fn get_mails(config: &Configuration) -> Result<Vec<Vec<u8>>> {
    let mut mails = Vec::new();
    let tls = TlsConnector::builder()
        .build()
        .context("Failed to build TLS connector")?;
    let addr = (config.imap_host.as_str(), config.imap_port);
    let client =
        imap::connect(addr, &config.imap_host, &tls).context("Failed to connect to IMAP server")?;
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
    let archive = ZipArchive::new(&zip_bytes).context("Failed to open body as ZIP")?;
    let mut reports = Vec::new();
    for file in archive.entries() {
        println!("File Name: -{}-", file.path);
        if !file.path.to_string().ends_with(".xml") {
            println!("Not an XML file, skipping...");
            continue;
        }
        let mut reader = archive.read(file).context("Failed to read file from ZIP")?;
        let report = dmarc_aggregate_parser::parse_reader(&mut reader)
            .context("Failed to parse XML as DMARC report")?;
        reports.push(report);
    }
    Ok(reports)
}

fn main() -> Result<()> {
    let config = Configuration::parse();

    let mails = get_mails(&config).context("Failed to get mails")?;
    println!("Downloaded {} mails", mails.len());

    for mail in mails {
        let reports = extract_reports(&mail).context("Failed to extract reports")?;
        for report in reports {
            println!("Report: {report:#?}");
        }
    }

    Ok(())
}
