use crate::geolocate::Location;
use crate::{cache_map::CacheMap, mail::Mail};
use crate::{dmarc, tls};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

const CACHE_SIZE: usize = 10000;

/// DMARC report with ID of the mail that contained the report
#[derive(Serialize, Deserialize)]
pub struct DmarcReportWithMailId {
    pub mail_id: String,
    pub report: dmarc::Report,
}

/// SMTP TLS report with ID of the mail that contained the report
#[derive(Serialize, Deserialize)]
pub struct TlsReportWithMailId {
    pub mail_id: String,
    pub report: tls::Report,
}

/// The type of a file that can contain report data
#[derive(Serialize, PartialEq)]
pub enum FileType {
    Json,
    Xml,
}

/// Parsing errors for DMARC or SMTP TLS reports
#[derive(Serialize)]
pub struct ReportParsingError {
    pub error: String,
    pub report: String,
    pub kind: FileType,
}

/// Shared state between the different parts of the application.
/// Connects the background task that collects mails via IMAP,
/// parses them, analyzes DMARC reports and makes them available for
/// the web frontend running on to the embedded HTTP server.
pub struct AppState {
    /// True until the first update after the application start finished
    pub first_update: bool,

    /// Mails from IMAP inbox with mail ID as key
    pub mails: HashMap<String, Mail>,

    /// Parsed DMARC reports with mail UID and corresponding hash as key
    pub dmarc_reports: HashMap<String, DmarcReportWithMailId>,

    /// Parsed SMTP TLS reports with mail UID and corresponding hash as key
    pub tls_reports: HashMap<String, TlsReportWithMailId>,

    /// Number of XML files extracted from mails
    pub xml_files: usize,

    /// Number of JSON files extracted from mails
    pub json_files: usize,

    /// Time of last update from IMAP inbox as Unix timestamp
    pub last_update: u64,

    /// XML DMARC and JSON SMTP TLS parsing errors keyed by mail ID
    pub parsing_errors: HashMap<String, Vec<ReportParsingError>>,

    /// IP to DNS cache
    pub ip_dns_cache: CacheMap<IpAddr, String>,

    /// IP to location cache
    pub ip_location_cache: CacheMap<IpAddr, Location>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            first_update: true,
            mails: HashMap::new(),
            dmarc_reports: HashMap::new(),
            tls_reports: HashMap::new(),
            last_update: 0,
            xml_files: 0,
            json_files: 0,
            parsing_errors: HashMap::new(),
            ip_dns_cache: CacheMap::new(CACHE_SIZE).expect("Failed to create DNS cache"),
            ip_location_cache: CacheMap::new(CACHE_SIZE).expect("Failed to create location cache"),
        }
    }
}
