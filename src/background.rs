use crate::config::Configuration;
use crate::hasher::create_hash;
use crate::imap::get_mails;
use crate::state::{
    AppState, DmarcReportWithMailId, FileType, ReportParsingError, TlsReportWithMailId,
};
use crate::unpack::extract_report_files;
use crate::web_hook::mail_web_hook;
use crate::{dmarc, tls};
use anyhow::{Context, Result};
use chrono::Local;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, trace, warn};

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
                Ok(new_mails) => {
                    info!("Detected {} new mails", new_mails.len());
                    info!(
                        "Finished background update after {:.3}s",
                        start.elapsed().as_secs_f64()
                    );
                    if !new_mails.is_empty() && config.mail_web_hook_url.is_some() {
                        debug!("Calling web hook for new mails...");
                        for mail_id in &new_mails {
                            if let Err(err) = mail_web_hook(&config, mail_id).await {
                                warn!("Failed to call web hook for mail {mail_id}: {err:#}");
                            }
                        }
                        debug!("Finished calling web hook for new mails");
                    }
                }
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

/// Executes a background update and returns the IDs of all new mails
async fn bg_update(config: &Configuration, state: &Arc<Mutex<AppState>>) -> Result<Vec<String>> {
    let mut mails = HashMap::new();
    if let Some(dmarc_folder) = config.imap_folder_dmarc.as_ref() {
        mails.extend(
            get_mails(config, dmarc_folder)
                .await
                .context("Failed to get mails from DMARC folder")?,
        );
    }
    if let Some(tls_folder) = config.imap_folder_tls.as_ref() {
        mails.extend(
            get_mails(config, tls_folder)
                .await
                .context("Failed to get mails from TLS folder")?,
        );
    }
    if config.imap_folder_dmarc.is_none() && config.imap_folder_tls.is_none() {
        mails.extend(
            get_mails(config, &config.imap_folder)
                .await
                .context("Failed to get mails")?,
        );
    }

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
        match extract_report_files(mail, config) {
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

    let mut parsing_errors: HashMap<String, Vec<ReportParsingError>> = HashMap::new();
    let mut dmarc_reports = HashMap::new();
    let mut tls_reports = HashMap::new();
    for xml_file in xml_files.values() {
        match dmarc::Report::from_slice(&xml_file.data) {
            Ok(report) => {
                let rwi = DmarcReportWithMailId {
                    report,
                    mail_id: xml_file.mail_id.clone(),
                };
                let hash = create_hash(&[&xml_file.data, xml_file.mail_id.as_bytes()]);
                dmarc_reports.insert(hash, rwi);
            }
            Err(err) => {
                // Prepare error information
                let error_str = format!("{err:#}");
                let error = ReportParsingError {
                    error: error_str,
                    report: String::from_utf8_lossy(&xml_file.data).to_string(),
                    kind: FileType::Xml,
                };

                // Store in error hashmap for fast lookup
                parsing_errors
                    .entry(xml_file.mail_id.clone())
                    .or_default()
                    .push(error);

                // Increase error counter for mail
                let mail = mails
                    .get_mut(&xml_file.mail_id)
                    .context("Failed to find mail")?;
                mail.xml_parsing_errors += 1;
            }
        }
    }

    for json_file in json_files.values() {
        match tls::Report::from_slice(&json_file.data) {
            Ok(report) => {
                let rwi = TlsReportWithMailId {
                    report,
                    mail_id: json_file.mail_id.clone(),
                };
                let hash = create_hash(&[&json_file.data, json_file.mail_id.as_bytes()]);
                tls_reports.insert(hash, rwi);
            }
            Err(err) => {
                // Prepare error information
                let error_str = format!("{err:#}");
                let error = ReportParsingError {
                    error: error_str,
                    report: String::from_utf8_lossy(&json_file.data).to_string(),
                    kind: FileType::Json,
                };

                // Store in error hashmap for fast lookup
                parsing_errors
                    .entry(json_file.mail_id.clone())
                    .or_default()
                    .push(error);

                // Increase error counter for mail
                let mail = mails
                    .get_mut(&json_file.mail_id)
                    .context("Failed to find mail")?;
                mail.json_parsing_errors += 1;
            }
        }
    }
    if !parsing_errors.is_empty() {
        warn!(
            "Failed to parse {} XML or JSON file(s) as DMARC or SMTP TLS report(s)",
            parsing_errors.len()
        );
    }

    info!(
        "Parsed {} DMARC reports and {} SMTP TLS reports successfully",
        dmarc_reports.len(),
        tls_reports.len()
    );

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("Failed to get Unix time stamp")?
        .as_secs();

    let new_mails = {
        let mut locked_state = state.lock().await;

        // Remember the IDs of all current mails from before the update
        let old_mails: HashSet<String> = locked_state.mails.keys().cloned().collect();

        // Update state with new values
        locked_state.dmarc_reports = dmarc_reports;
        locked_state.tls_reports = tls_reports;
        locked_state.last_update = timestamp;
        locked_state.xml_files = xml_files.len();
        locked_state.json_files = json_files.len();
        locked_state.parsing_errors = parsing_errors;
        locked_state.mails = mails;

        // Detect which of the mails are new
        let new_mails: Vec<String> = locked_state.mails.keys().cloned().collect();
        if locked_state.first_update {
            locked_state.first_update = false;

            // During the intial update we do not report any mails as new
            vec![]
        } else {
            new_mails
                .into_iter()
                .filter(|id| !old_mails.contains(id))
                .collect()
        }
    };

    Ok(new_mails)
}
