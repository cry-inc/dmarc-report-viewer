use crate::dmarc::DkimResultType;
use crate::dmarc::DmarcResultType;
use crate::dmarc::RecordType;
use crate::dmarc::SpfResultType;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, PartialEq, Eq, Hash)]
enum SourceIssue {
    SpfPolicy,
    DkimPolicy,
    SpfAuth,
    DkimAuth,
}

#[derive(Serialize)]
struct SourceDetails {
    count: usize,
    domain: String,
    issues: HashSet<SourceIssue>,
}

#[derive(Serialize)]
struct Source {
    ip: IpAddr,
    #[serde(flatten)]
    details: SourceDetails,
}

pub async fn handler(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let mut ip_map = HashMap::new();
    {
        let locked_state = state.lock().await;
        for report in locked_state.dmarc_reports.values() {
            for record in &report.report.record {
                // Get or create details for IP
                let details = ip_map.entry(record.row.source_ip).or_insert(SourceDetails {
                    count: 0,
                    domain: report.report.policy_published.domain.clone(),
                    issues: HashSet::new(),
                });

                // Update count
                details.count += record.row.count;

                // Detect any issues
                detect_dmarc_issues(record, &mut details.issues);
            }
        }
    }

    let mut sources: Vec<Source> = ip_map
        .into_iter()
        .map(|(ip, details)| Source { ip, details })
        .collect();

    // Sort descending by count
    sources.sort_by(|a, b| b.details.count.cmp(&a.details.count));

    let json = serde_json::to_string(&sources).expect("Failed to serialize sources as JSON");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        json,
    )
}

fn detect_dmarc_issues(record: &RecordType, issues: &mut HashSet<SourceIssue>) {
    if let Some(dkim) = &record.row.policy_evaluated.dkim {
        if *dkim != DmarcResultType::Pass {
            issues.insert(SourceIssue::DkimPolicy);
        }
    }
    if let Some(spf) = &record.row.policy_evaluated.spf {
        if *spf != DmarcResultType::Pass {
            issues.insert(SourceIssue::SpfPolicy);
        }
    }
    if let Some(dkim) = &record.auth_results.dkim {
        if dkim.iter().any(|x| x.result != DkimResultType::Pass) {
            issues.insert(SourceIssue::DkimAuth);
        }
    }
    if record
        .auth_results
        .spf
        .iter()
        .any(|x| x.result != SpfResultType::Pass)
    {
        issues.insert(SourceIssue::SpfAuth);
    }
}
