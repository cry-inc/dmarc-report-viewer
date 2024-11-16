use std::collections::HashMap;

use crate::mail::Mail;
use crate::report::Report;
use crate::summary::Summary;
use crate::xml_error::XmlError;

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

    /// Parsed DMARC reports as tuple of mail UID and report
    pub reports: Vec<(u32, Report)>,

    /// Summary of report and other stats
    pub summary: Summary,

    /// Time of last update from IMAP inbox as Unix timestamp
    pub last_update: u64,

    /// XML parsing errors
    pub xml_errors: Vec<XmlError>,
}
