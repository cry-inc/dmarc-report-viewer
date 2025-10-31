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
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize)]
struct ReportHeader {
    hash: String,
    id: String,
    org: String,
    domain: String,
    date_begin: u64,
    date_end: u64,
    records: usize,
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
            flagged: flagged_dkim | flagged_spf | flagged_dmarc,
            flagged_dkim,
            flagged_spf,
            flagged_dmarc,
        }
    }

    /// Returns if the report has DKIM or SPF issues
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
	        if !matches!(record.row.policy_evaluated.dkim, Some(DmarcResultType::Pass))
    		&& !matches!(record.row.policy_evaluated.spf,  Some(DmarcResultType::Pass))
	        {
                dmarc_flagged = true;
            }
            if let Some(dkim) = &record.auth_results.dkim
                && dkim.iter().any(|x| x.result != DkimResultType::Pass)
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
        }
        (dkim_flagged, spf_flagged, dmarc_flagged)
    }
}

#[derive(Deserialize)]
pub struct ReportFilters {
    id: Option<String>,
    flagged: Option<bool>,
    flagged_dkim: Option<bool>,
    flagged_spf: Option<bool>,
    flagged_dmarc: Option<bool>,
    domain: Option<String>,
    org: Option<String>,
    ip: Option<String>,
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

pub async fn list_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    mut filters: Query<ReportFilters>,
) -> impl IntoResponse {
    // Remove URL encoding from strings in filters
    filters.url_decode();

    // Parse IP once to speed up filters
    let ip_filter = filters.ip.as_deref().and_then(|s| IpAddr::from_str(s).ok());

    let reports: Vec<ReportHeader> = state
        .lock()
        .await
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
    Json(reports)
}

pub async fn single_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let lock = state.lock().await;
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
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let lock = state.lock().await;
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
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let lock = state.lock().await;
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
