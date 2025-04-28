use crate::config::Configuration;
use crate::dmarc::{DmarcParsingError, Report};
use crate::hasher::hash_data;
use crate::imap::get_mails;
use crate::state::{AppState, DmarcReportWithUid};
use crate::summary::Summary;
use crate::unpack::extract_xml_files;
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info, trace, warn};

pub fn start_bg_task(
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
            let start = Instant::now();
            info!("Starting background update...");
            match bg_update(&config, &state).await {
                Ok(..) => info!(
                    "Finished background update after {:.3}s",
                    start.elapsed().as_secs_f64()
                ),
                Err(err) => error!("Failed background update: {err:#}"),
            };

            // Check how many seconds we need to sleep
            let mut duration = Duration::from_secs(config.imap_check_interval);
            if let Some(schedule) = &config.imap_check_schedule {
                if let Some(next_update) = schedule.upcoming(Utc).next() {
                    let delta = next_update - Utc::now();
                    duration = Duration::from_millis(delta.num_milliseconds().max(0) as u64)
                } else {
                    warn!("Unable to find next scheduled check, falling back to interval...")
                }
            }

            // Print next update time
            let next = Utc::now() + duration;
            info!("Next update is planned for {next}");

            tokio::select! {
                _ = tokio::time::sleep(duration) => {},
                _ = stop_signal.recv() => { break; },
            }
        }
    })
}

async fn bg_update(config: &Configuration, state: &Arc<Mutex<AppState>>) -> Result<()> {
    let mut mails = get_mails(config).await.context("Failed to get mails")?;

    let mut xml_files = HashMap::new();
    let mut mails_without_xml = 0;
    for mail in &mut mails.values_mut() {
        if mail.body.is_none() {
            trace!(
                "Skipping data extraction for mail with UID {} because of empty body",
                mail.uid
            );
            continue;
        }
        match extract_xml_files(mail) {
            Ok(files) => {
                if files.is_empty() {
                    mails_without_xml += 1;
                }
                for xml_file in files {
                    xml_files.insert(xml_file.hash.clone(), xml_file);
                    mail.xml_files += 1;
                }
            }
            Err(err) => warn!("Failed to extract XML files from mail: {err:#}"),
        }
    }
    if mails_without_xml > 0 {
        warn!("Found {mails_without_xml} mail(s) without XML files");
    }
    info!("Extracted {} XML file(s)", xml_files.len());

    let mut dmarc_parsing_errors = HashMap::new();
    let mut dmarc_reports = HashMap::new();
    for xml_file in xml_files.values() {
        match Report::from_slice(&xml_file.data) {
            Ok(report) => {
                let rwu = DmarcReportWithUid {
                    report,
                    uid: xml_file.mail_uid,
                };
                let binary =
                    serde_json::to_vec(&rwu).context("Failed to serialize report with UID")?;
                let hash = hash_data(&binary);
                dmarc_reports.insert(hash, rwu);
            }
            Err(err) => {
                // Prepare error information
                let error_str = format!("{err:#}");
                let error = DmarcParsingError {
                    error: error_str,
                    xml: String::from_utf8_lossy(&xml_file.data).to_string(),
                };

                // Store in error hashmap for fast lookup
                let entry: &mut Vec<DmarcParsingError> =
                    dmarc_parsing_errors.entry(xml_file.mail_uid).or_default();
                entry.push(error);

                // Increase error counter for mail
                let mail = mails
                    .get_mut(&xml_file.mail_uid)
                    .context("Failed to find mail")?;
                mail.parsing_errors += 1;
            }
        }
    }
    info!("Parsed {} DMARC reports successfully", dmarc_reports.len());
    if !dmarc_parsing_errors.is_empty() {
        warn!(
            "Failed to parse {} XML file as DMARC reports",
            dmarc_parsing_errors.len()
        );
    }

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("Failed to get Unix time stamp")?
        .as_secs();

    let summary = Summary::new(mails.len(), xml_files.len(), &dmarc_reports, timestamp);

    {
        let mut locked_state = state.lock().await;
        locked_state.mails = mails;
        locked_state.summary = summary;
        locked_state.dmarc_reports = dmarc_reports;
        locked_state.last_update = timestamp;
        locked_state.dmarc_parsing_errors = dmarc_parsing_errors;
    }

    Ok(())
}
