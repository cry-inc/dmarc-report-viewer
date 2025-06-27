use crate::state::AppState;
use crate::tls::PolicyType;
use crate::tls::Report;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::header;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize)]
struct ReportHeader {
    hash: String,
    id: String,
    org: String,
    domains: Vec<String>,
    date_begin: DateTime<Utc>,
    date_end: DateTime<Utc>,
    records: usize,
    flagged_sts: bool,
    flagged_tlsa: bool,
    flagged: bool,
}

impl ReportHeader {
    pub fn from_report(hash: &str, report: &Report) -> Self {
        let (flagged_sts, flagged_tlsa) = Self::flags(report);
        Self {
            hash: hash.to_string(),
            id: report.report_id.clone(),
            org: report.organization_name.clone(),
            domains: {
                let mut domains = report
                    .policies
                    .iter()
                    .map(|p| p.policy.policy_domain.clone())
                    .collect::<Vec<String>>();
                domains.sort();
                domains.dedup();
                domains
            },
            date_begin: report.date_range.start_datetime,
            date_end: report.date_range.end_datetime,
            records: report.policies.len(),
            flagged: flagged_sts || flagged_tlsa,
            flagged_sts,
            flagged_tlsa,
        }
    }

    /// Returns if the report has STS or TLSA issues
    fn flags(report: &Report) -> (bool, bool) {
        let mut sts_flagged = false;
        let mut tlsa_flagged = false;
        for policy_result in &report.policies {
            if policy_result.summary.total_failure_session_count > 0 {
                if policy_result.policy.policy_type == PolicyType::Sts {
                    sts_flagged = true;
                } else if policy_result.policy.policy_type == PolicyType::Tlsa {
                    tlsa_flagged = true;
                }
            }
        }
        (sts_flagged, tlsa_flagged)
    }
}

#[derive(Deserialize)]
pub struct ReportFilters {
    id: Option<String>,
    flagged: Option<bool>,
    flagged_sts: Option<bool>,
    flagged_tlsa: Option<bool>,
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
        .tls_reports
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
                rwi.report.organization_name == *org
            } else {
                true
            }
        })
        .filter(|(_, rwi)| {
            if let Some(domain) = &filters.domain {
                rwi.report
                    .policies
                    .iter()
                    .any(|policy_result| policy_result.policy.policy_domain == *domain)
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
            if let Some(sts) = &filters.flagged_sts {
                rh.flagged_sts == *sts
            } else {
                true
            }
        })
        .filter(|rh| {
            if let Some(tlsa) = &filters.flagged_tlsa {
                rh.flagged_tlsa == *tlsa
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
    if let Some(rwi) = lock.tls_reports.get(&id) {
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
    if let Some(rwi) = lock.tls_reports.get(&id) {
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
