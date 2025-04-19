use crate::report::DkimResultType;
use crate::report::DmarcResultType;
use crate::report::Report;
use crate::report::SpfResultType;
use crate::state::AppState;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::header;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
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
        let (flagged_dkim, flagged_spf) = Self::report_is_flagged(report);
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

    /// Checks if the report has DKIM or SPF issues
    fn report_is_flagged(report: &Report) -> (bool, bool) {
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
    uid: Option<u32>,
    flagged: Option<bool>,
    flagged_dkim: Option<bool>,
    flagged_spf: Option<bool>,
    domain: Option<String>,
    org: Option<String>,
}

impl ReportFilters {
    fn url_decode(&self) -> Self {
        Self {
            uid: self.uid,
            flagged: self.flagged,
            flagged_dkim: self.flagged_dkim,
            flagged_spf: self.flagged_spf,
            domain: self
                .domain
                .as_ref()
                .and_then(|d| urlencoding::decode(d).ok())
                .map(|d| d.to_string()),
            org: self
                .org
                .as_ref()
                .and_then(|o| urlencoding::decode(o).ok())
                .map(|o| o.to_string()),
        }
    }
}

pub async fn list_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    filters: Query<ReportFilters>,
) -> impl IntoResponse {
    // Remove URL encoding from strings in filters
    let filters = filters.url_decode();

    let reports: Vec<ReportHeader> = state
        .lock()
        .await
        .reports
        .iter()
        .filter(|(_, rwu)| {
            if let Some(queried_uid) = filters.uid {
                rwu.uid == queried_uid
            } else {
                true
            }
        })
        .filter(|(_, rwu)| {
            if let Some(org) = &filters.org {
                rwu.report.report_metadata.org_name == *org
            } else {
                true
            }
        })
        .filter(|(_, rwu)| {
            if let Some(domain) = &filters.domain {
                rwu.report.policy_published.domain == *domain
            } else {
                true
            }
        })
        .map(|(hash, rwu)| ReportHeader::from_report(hash, &rwu.report))
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
    if let Some(rwu) = lock.reports.get(&id) {
        let report_json = serde_json::to_string(rwu).expect("Failed to serialize JSON");
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            report_json,
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            format!("Cannot find report with ID {id}"),
        )
    }
}

pub async fn json_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let lock = state.lock().await;
    if let Some(rwu) = lock.reports.get(&id) {
        let report_json = serde_json::to_string(&rwu.report).expect("Failed to serialize JSON");
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            report_json,
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            format!("Cannot find report with ID {id}"),
        )
    }
}

pub async fn xml_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let lock = state.lock().await;
    if let Some(rwu) = lock.reports.get(&id) {
        let mut report_xml = String::new();
        let mut serializer = quick_xml::se::Serializer::new(&mut report_xml);
        serializer.indent(' ', 2);
        rwu.report
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
            format!("Cannot find report with ID {id}"),
        )
    }
}
