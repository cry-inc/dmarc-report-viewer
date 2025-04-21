use crate::dmarc::{DmarcParsingError, Report};
use crate::geolocate::Location;
use crate::summary::Summary;
use crate::{cache_map::CacheMap, mail::Mail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

const CACHE_SIZE: usize = 10000;

/// Report with UID of the mail that contained the report
#[derive(Debug, Serialize, Deserialize)]
pub struct DmarcReportWithUid {
    pub uid: u32,
    pub report: Report,
}

/// Shared state between the different parts of the application.
/// Connects the background task that collects mails via IMAP,
/// parses them, analyzes DMARC reports and makes them available for
/// the web frontend running on to the embedded HTTP server.
pub struct AppState {
    /// Mails from IMAP inbox with mail UID as key
    pub mails: HashMap<u32, Mail>,

    /// Parsed DMARC reports with mail UID and corresponding hash as key
    pub dmarc_reports: HashMap<String, DmarcReportWithUid>,

    /// Summary of report and other stats
    pub summary: Summary,

    /// Time of last update from IMAP inbox as Unix timestamp
    pub last_update: u64,

    /// XML DMARC parsing errors keyed by mail UID
    pub dmarc_parsing_errors: HashMap<u32, Vec<DmarcParsingError>>,

    /// IP to DNS cache
    pub ip_dns_cache: CacheMap<IpAddr, String>,

    /// IP to location cache
    pub ip_location_cache: CacheMap<IpAddr, Location>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mails: HashMap::new(),
            dmarc_reports: HashMap::new(),
            summary: Summary::default(),
            last_update: 0,
            dmarc_parsing_errors: HashMap::new(),
            ip_dns_cache: CacheMap::new(CACHE_SIZE).expect("Failed to create DNS cache"),
            ip_location_cache: CacheMap::new(CACHE_SIZE).expect("Failed to create location cache"),
        }
    }
}
