use dmarc_aggregate_parser::aggregate_report::feedback;

/// Shared state between the different parts of the application.
/// Connects the background task that collects mails via IMAP,
/// parses them, analyzes DMARC reports and makes them available for
/// the web frontend running on to the embedded HTTP server.
#[derive(Default)]
pub struct AppState {
    /// Number of emails in IMAP report inbox
    pub mails: usize,

    /// DMARC reports parsed from emails in inbox
    pub reports: Vec<feedback>,
}
