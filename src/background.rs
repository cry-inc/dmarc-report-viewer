use crate::config::Configuration;
use crate::hasher::create_hash;
use crate::imap::get_mails;
use crate::state::{AppState, DmarcReportWithUid, ReportParsingError, TlsRptReportWithUid};
use crate::unpack::{extract_report_files, FileType};
use crate::{dmarc, tlsrpt};
use anyhow::{Context, Result};
use chrono::Local;
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
                if let Some(next_update) = schedule.upcoming(Local).next() {
                    let delta = next_update - Local::now();
                    duration = Duration::from_millis(delta.num_milliseconds().max(0) as u64)
                } else {
                    warn!("Unable to find next scheduled check, falling back to interval...")
                }
            }

            // Print next update time
            let next = Local::now() + duration;
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
    let mut json_files = HashMap::new();
    let mut mails_without_reports = 0;
    for mail in &mut mails.values_mut() {
        if mail.body.is_none() {
            trace!(
                "Skipping data extraction for mail with UID {} because of empty body",
                mail.uid
            );
            continue;
        }
        match extract_report_files(mail) {
            Ok(files) => {
                if files.is_empty() {
                    mails_without_reports += 1;
                }
                for file in files {
                    match file.file_type {
                        FileType::Xml => {
                            xml_files.insert(file.hash.clone(), file);
                            mail.xml_files += 1;
                        }
                        FileType::Json => {
                            json_files.insert(file.hash.clone(), file);
                            mail.json_files += 1;
                        }
                    }
                }
            }
            Err(err) => warn!("Failed to extract report files from mail: {err:#}"),
        }
    }
    if mails_without_reports > 0 {
        warn!("Found {mails_without_reports} mail(s) without report files");
    }
    info!(
        "Extracted {} XML report file(s) and {} JSON report file(s)",
        xml_files.len(),
        json_files.len()
    );

    let mut dmarc_parsing_errors = HashMap::new();
    let mut dmarc_reports = HashMap::new();
    let mut tlsrpt_parsing_errors = HashMap::new();
    let mut tlsrpt_reports = HashMap::new();
    for xml_file in xml_files.values() {
        match dmarc::Report::from_slice(&xml_file.data) {
            Ok(report) => {
                let rwu = DmarcReportWithUid {
                    report,
                    uid: xml_file.mail_uid,
                };
                let hash = create_hash(&xml_file.data, Some(xml_file.mail_uid));
                dmarc_reports.insert(hash, rwu);
            }
            Err(err) => {
                // Prepare error information
                let error_str = format!("{err:#}");
                let error = ReportParsingError {
                    error: error_str,
                    report: String::from_utf8_lossy(&xml_file.data).to_string(),
                };

                // Store in error hashmap for fast lookup
                let entry: &mut Vec<_> = dmarc_parsing_errors.entry(xml_file.mail_uid).or_default();
                entry.push(error);

                // Increase error counter for mail
                let mail = mails
                    .get_mut(&xml_file.mail_uid)
                    .context("Failed to find mail")?;
                mail.xml_parsing_errors += 1;
            }
        }
    }
    if !dmarc_parsing_errors.is_empty() {
        warn!(
            "Failed to parse {} XML file as DMARC reports",
            dmarc_parsing_errors.len()
        );
    }

    for json_file in json_files.values() {
        match tlsrpt::Report::from_slice(&json_file.data) {
            Ok(report) => {
                let rwu = TlsRptReportWithUid {
                    report,
                    uid: json_file.mail_uid,
                };
                let hash = create_hash(&json_file.data, Some(json_file.mail_uid));
                tlsrpt_reports.insert(hash, rwu);
            }
            Err(err) => {
                // Prepare error information
                let error_str = format!("{err:#}");
                let error = ReportParsingError {
                    error: error_str,
                    report: String::from_utf8_lossy(&json_file.data).to_string(),
                };

                // Store in error hashmap for fast lookup
                let entry: &mut Vec<_> =
                    tlsrpt_parsing_errors.entry(json_file.mail_uid).or_default();
                entry.push(error);

                // Increase error counter for mail
                let mail = mails
                    .get_mut(&json_file.mail_uid)
                    .context("Failed to find mail")?;
                mail.json_parsing_errors += 1;
            }
        }
    }
    if !tlsrpt_parsing_errors.is_empty() {
        warn!(
            "Failed to parse {} JSON file as TLS-RPT reports",
            tlsrpt_parsing_errors.len()
        );
    }

    info!(
        "Parsed {} DMARC reports and {} TLS-RPT reports successfully",
        dmarc_reports.len(),
        tlsrpt_reports.len()
    );

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("Failed to get Unix time stamp")?
        .as_secs();

    {
        let mut locked_state = state.lock().await;
        locked_state.mails = mails;
        locked_state.dmarc_reports = dmarc_reports;
        locked_state.tlsrpt_reports = tlsrpt_reports;
        locked_state.last_update = timestamp;
        locked_state.xml_files = xml_files.len();
        locked_state.json_files = json_files.len();
        locked_state.dmarc_parsing_errors = dmarc_parsing_errors;
        locked_state.tlsrpt_parsing_errors = tlsrpt_parsing_errors;
    }

    Ok(())
}
