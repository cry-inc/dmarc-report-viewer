use anyhow::{Context, Result};
use dmarc_aggregate_parser::aggregate_report::feedback;
use std::io::Cursor;
use tracing::warn;
use zip::ZipArchive;

pub fn extract_reports(mail: &[u8]) -> Result<Vec<feedback>> {
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
