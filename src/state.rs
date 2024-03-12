use crate::dmarc_report::Report;
use crate::summary::Summary;

/// Shared state between the different parts of the application.
/// Connects the background task that collects mails via IMAP,
/// parses them, analyzes DMARC reports and makes them available for
/// the web frontend running on to the embedded HTTP server.
#[derive(Default)]
pub struct AppState {
    /// Number of emails in IMAP report inbox
    pub mails: usize,

    /// Number of XML files found in IMAP report inbox
    pub xml_files: usize,

    /// DMARC reports parsed from emails in inbox
    pub reports: Vec<Report>,

    /// Summary of report and other stats
    pub summary: Summary,

    /// Time of last update from IMAP inbox as Unix timestamp
    pub last_update: u64,
}
