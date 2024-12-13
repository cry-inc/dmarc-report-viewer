use crate::mail::Mail;
use crate::report::Report;
use crate::summary::Summary;
use crate::xml_error::XmlError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Report with UID of the mail that contained the report
#[derive(Debug, Serialize, Deserialize)]
pub struct ReportWithUid {
    pub uid: u32,
    pub report: Report,
}

/// Shared state between the different parts of the application.
/// Connects the background task that collects mails via IMAP,
/// parses them, analyzes DMARC reports and makes them available for
/// the web frontend running on to the embedded HTTP server.
#[derive(Default)]
pub struct AppState {
    /// Mails from IMAP inbox with mail UID as key
    pub mails: HashMap<u32, Mail>,

    /// Number of XML files found in IMAP report inbox
    pub xml_files: usize,

    /// Parsed DMARC reports with mail UID and corresponding hash as key
    pub reports: HashMap<String, ReportWithUid>,

    /// Summary of report and other stats
    pub summary: Summary,

    /// Time of last update from IMAP inbox as Unix timestamp
    pub last_update: u64,

    /// XML parsing errors
    pub xml_errors: Vec<XmlError>,
}
