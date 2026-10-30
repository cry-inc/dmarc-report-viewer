use crate::dmarc::DkimResultType;
use crate::dmarc::DmarcResultType;
use crate::dmarc::Report;
use crate::dmarc::SpfResultType;
use crate::state::AppState;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Serialize)]
struct ReportHeader {
    hash: String,
    id: String,
    org: String,
    domain: String,
    date_begin: u64,
    date_end: u64,
    records: usize,
    count: usize,
    flagged_dkim: bool,
    flagged_spf: bool,
    flagged_dmarc: bool,
    flagged: bool,
}

impl ReportHeader {
    pub fn from_report(hash: &str, report: &Report) -> Self {
        let (flagged_dkim, flagged_spf, flagged_dmarc) = Self::flags(report);
        Self {
            hash: hash.to_string(),
            id: report.report_metadata.report_id.clone(),
            org: report.report_metadata.org_name.clone(),
            domain: report.policy_published.domain.clone(),
            date_begin: report.report_metadata.date_range.begin,
            date_end: report.report_metadata.date_range.end,
            records: report.record.len(),
            count: report.record.iter().map(|r| r.row.count).sum(),
            flagged: flagged_dkim | flagged_spf | flagged_dmarc,
            flagged_dkim,
            flagged_spf,
            flagged_dmarc,
        }
    }

    /// Returns if the report has DKIM, SPF or DMARC issues
    fn flags(report: &Report) -> (bool, bool, bool) {
        let mut dkim_flagged = false;
        let mut spf_flagged = false;
        let mut dmarc_flagged = false;
        for record in &report.record {
            if let Some(dkim) = &record.row.policy_evaluated.dkim
                && *dkim != DmarcResultType::Pass
            {
                dkim_flagged = true;
            }
            if let Some(spf) = &record.row.policy_evaluated.spf
                && *spf != DmarcResultType::Pass
            {
                spf_flagged = true;
            }
            // One passing signature is enough, mails can carry multiple
            // signatures (e.g. ed25519 + RSA) and receivers may not support all.
            if let Some(dkim) = &record.auth_results.dkim
                && !dkim.is_empty()
                && dkim.iter().all(|x| x.result != DkimResultType::Pass)
            {
                dkim_flagged = true;
            }
            if record
                .auth_results
                .spf
                .iter()
                .any(|x| x.result != SpfResultType::Pass)
            {
                spf_flagged = true;
            }
            let dkim_eval_pass = matches!(
                record.row.policy_evaluated.dkim,
                Some(DmarcResultType::Pass)
            );
            let spf_eval_pass =
                matches!(record.row.policy_evaluated.spf, Some(DmarcResultType::Pass));
            if !dkim_eval_pass && !spf_eval_pass {
                dmarc_flagged = true;
            }
        }
        (dkim_flagged, spf_flagged, dmarc_flagged)
    }
}

#[derive(Deserialize)]
pub struct ReportFilters {
    /// Report hash or ID of the mail that contains the report.
    id: Option<String>,
    /// Whether the report contains any DKIM, SPF, or DMARC issues.
    flagged: Option<bool>,
    /// Whether the report contains DKIM issues.
    flagged_dkim: Option<bool>,
    /// Whether the report contains SPF issues.
    flagged_spf: Option<bool>,
    /// Whether the report contains DMARC issues.
    flagged_dmarc: Option<bool>,
    /// Exact domain to filter reports by.
    domain: Option<String>,
    /// Organization name reported by the sender.
    org: Option<String>,
    /// Source IP address to find in the report's records.
    ip: Option<String>,
    /// Unix timestamp in seconds.
    /// If set, reports ending before this are excluded.
    after: Option<i64>,
    /// Unix timestamp in seconds.
    /// If set, reports beginning after this are excluded.
    before: Option<i64>,
}

impl ReportFilters {
    fn url_decode(&mut self) {
        self.domain = self
            .domain
            .as_ref()
            .and_then(|d| urlencoding::decode(d).ok())
            .map(|d| d.to_lowercase());
        self.org = self
            .org
            .as_ref()
            .and_then(|o| urlencoding::decode(o).ok())
            .map(|o| o.to_string());
        self.ip = self
            .ip
            .as_ref()
            .and_then(|i| urlencoding::decode(i).ok())
            .map(|i| i.to_string());
    }
}

#[derive(Serialize)]
struct ReportListResponse {
    /// Requested filtered reports
    reports: Vec<ReportHeader>,
    /// Domains from all reports (unfiltered)
    domains: Vec<String>,
    /// Orgs from all reports (unfiltered)
    orgs: Vec<String>,
}

pub async fn list_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    mut filters: Query<ReportFilters>,
) -> impl IntoResponse {
    // Remove URL encoding from strings in filters
    filters.url_decode();

    // Parse IP once to speed up filters
    let ip_filter = filters.ip.as_deref().and_then(|s| IpAddr::from_str(s).ok());

    let state = state.read().await;
    let domains = state
        .dmarc_reports
        .values()
        .map(|rwi| rwi.report.policy_published.domain.as_str())
        .collect::<HashSet<&str>>()
        .into_iter()
        .map(|d| d.to_string())
        .collect();
    let orgs = state
        .dmarc_reports
        .values()
        .map(|rwi| rwi.report.report_metadata.org_name.as_str())
        .collect::<HashSet<&str>>()
        .into_iter()
        .map(|o| o.to_string())
        .collect();
    let reports: Vec<ReportHeader> = state
        .dmarc_reports
        .iter()
        .filter(|(_, rwi)| {
            if let Some(id) = &filters.id {
                rwi.mail_id == *id
            } else {
                true
            }
        })
        .filter(|(_, rwi)| {
            if let Some(after) = filters.after {
                i128::from(rwi.report.report_metadata.date_range.end) >= i128::from(after)
            } else {
                true
            }
        })
        .filter(|(_, rwi)| {
            if let Some(before) = filters.before {
                i128::from(rwi.report.report_metadata.date_range.begin) <= i128::from(before)
            } else {
                true
            }
        })
        .filter(|(_, rwi)| {
            if let Some(org) = &filters.org {
                rwi.report.report_metadata.org_name == *org
            } else {
                true
            }
        })
        .filter(|(_, rwi)| {
            if let Some(fd) = &filters.domain {
                rwi.report.policy_published.domain.to_lowercase() == *fd
            } else {
                true
            }
        })
        .filter(|(_, rwi)| {
            if let Some(ip) = &ip_filter {
                rwi.report.record.iter().any(|r| r.row.source_ip == *ip)
            } else {
                true
            }
        })
        .map(|(hash, rwi)| ReportHeader::from_report(hash, &rwi.report))
        .filter(|rh| {
            if let Some(flagged) = &filters.flagged {
                rh.flagged == *flagged
            } else {
                true
            }
        })
        .filter(|rh| {
            if let Some(dkim) = &filters.flagged_dkim {
                rh.flagged_dkim == *dkim
            } else {
                true
            }
        })
        .filter(|rh| {
            if let Some(spf) = &filters.flagged_spf {
                rh.flagged_spf == *spf
            } else {
                true
            }
        })
        .filter(|rh| {
            if let Some(dm) = &filters.flagged_dmarc {
                rh.flagged_dmarc == *dm
            } else {
                true
            }
        })
        .collect();
    drop(state);

    Json(ReportListResponse {
        reports,
        domains,
        orgs,
    })
}

pub async fn single_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let lock = state.read().await;
    if let Some(rwi) = lock.dmarc_reports.get(&id) {
        let report_json = serde_json::to_string(rwi).expect("Failed to serialize JSON");
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            report_json,
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            String::from("Cannot find report"),
        )
    }
}

pub async fn json_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let lock = state.read().await;
    if let Some(rwi) = lock.dmarc_reports.get(&id) {
        let report_json = serde_json::to_string(&rwi.report).expect("Failed to serialize JSON");
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            report_json,
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            String::from("Cannot find report"),
        )
    }
}

pub async fn xml_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let lock = state.read().await;
    if let Some(rwi) = lock.dmarc_reports.get(&id) {
        let mut report_xml = String::new();
        let mut serializer = quick_xml::se::Serializer::new(&mut report_xml);
        serializer.indent(' ', 2);
        rwi.report
            .serialize(serializer)
            .expect("Failed to serialize XML");
        report_xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n") + &report_xml;
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/xml")],
            report_xml,
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            String::from("Cannot find report"),
        )
    }
}
