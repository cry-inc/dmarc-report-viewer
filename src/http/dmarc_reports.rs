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
    flagged: bool,
}

impl ReportHeader {
    pub fn from_report(hash: &str, report: &Report) -> Self {
        let (flagged_dkim, flagged_spf) = Self::flags(report);
        Self {
            hash: hash.to_string(),
            id: report.report_metadata.report_id.clone(),
            org: report.report_metadata.org_name.clone(),
            domain: report.policy_published.domain.clone(),
            date_begin: report.report_metadata.date_range.begin,
            date_end: report.report_metadata.date_range.end,
            records: report.record.len(),
            flagged: flagged_dkim | flagged_spf,
            flagged_dkim,
            flagged_spf,
        }
    }

    /// Returns if the report has DKIM or SPF issues
    fn flags(report: &Report) -> (bool, bool) {
        let mut dkim_flagged = false;
        let mut spf_flagged = false;
        for record in &report.record {
            if let Some(dkim) = &record.row.policy_evaluated.dkim {
                if *dkim != DmarcResultType::Pass {
                    dkim_flagged = true;
                }
            }
            if let Some(spf) = &record.row.policy_evaluated.spf {
                if *spf != DmarcResultType::Pass {
                    spf_flagged = true;
                }
            }
            if let Some(dkim) = &record.auth_results.dkim {
                if dkim.iter().any(|x| x.result != DkimResultType::Pass) {
                    dkim_flagged = true;
                }
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
        (dkim_flagged, spf_flagged)
    }
}

#[derive(Deserialize)]
pub struct ReportFilters {
    id: Option<String>,
    flagged: Option<bool>,
    flagged_dkim: Option<bool>,
    flagged_spf: Option<bool>,
    domain: Option<String>,
    org: Option<String>,
}

impl ReportFilters {
    fn url_decode(&mut self) {
        self.domain = self
            .domain
            .as_ref()
            .and_then(|d| urlencoding::decode(d).ok())
            .map(|d| d.to_string());
        self.org = self
            .org
            .as_ref()
            .and_then(|o| urlencoding::decode(o).ok())
            .map(|o| o.to_string());
    }
}

pub async fn list_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    mut filters: Query<ReportFilters>,
) -> impl IntoResponse {
    // Remove URL encoding from strings in filters
    filters.url_decode();

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
            if let Some(domain) = &filters.domain {
                rwi.report.policy_published.domain == *domain
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
