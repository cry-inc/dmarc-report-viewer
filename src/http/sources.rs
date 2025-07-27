use crate::dmarc::DkimResultType;
use crate::dmarc::DmarcResultType;
use crate::dmarc::RecordType;
use crate::dmarc::SpfResultType;
use crate::state::AppState;
use crate::tls::FailureResultType;
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
enum Issue {
    // DMARC
    SpfPolicy,
    DkimPolicy,
    SpfAuth,
    DkimAuth,

    // TLS
    StarttlsNotSupported,
    CertificateHostMismatch,
    CertificateExpired,
    CertificateNotTrusted,
    ValidationFailure,
    TlsaInvalid,
    DnssecInvalid,
    DaneRequired,
    StsPolicyFetchError,
    StsPolicyInvalid,
    StsWebpkiInvalid,
}

#[derive(Serialize, PartialEq, Eq, Hash)]
enum ReportType {
    Dmarc,
    Tls,
}

#[derive(Serialize)]
struct SourceDetails {
    count: usize,
    domain: String,
    issues: HashSet<Issue>,
    types: HashSet<ReportType>,
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

        // Get source IPs from DMARC reports
        for report in locked_state.dmarc_reports.values() {
            for record in &report.report.record {
                // Get or create details for IP
                let details = ip_map.entry(record.row.source_ip).or_insert(SourceDetails {
                    count: 0,
                    domain: report.report.policy_published.domain.clone(),
                    issues: HashSet::new(),
                    types: HashSet::new(),
                });

                // Update count
                details.count += record.row.count;

                // Detect any issues
                detect_dmarc_issues(record, &mut details.issues);

                // Add report type
                details.types.insert(ReportType::Dmarc);
            }
        }

        // get source IPs from TLS reports
        for report in locked_state.tls_reports.values() {
            for policy in &report.report.policies {
                let Some(failures) = &policy.failure_details else {
                    continue;
                };

                for failure in failures {
                    // Get or create details for IP
                    let details = ip_map
                        .entry(failure.sending_mta_ip)
                        .or_insert(SourceDetails {
                            count: 0,
                            domain: policy.policy.policy_domain.clone(),
                            issues: HashSet::new(),
                            types: HashSet::new(),
                        });

                    // Update count
                    details.count += failure.failed_session_count;

                    // Add issue type
                    details.issues.insert(match failure.result_type {
                        FailureResultType::StarttlsNotSupported => Issue::StarttlsNotSupported,
                        FailureResultType::CertificateHostMismatch => {
                            Issue::CertificateHostMismatch
                        }
                        FailureResultType::CertificateExpired => Issue::CertificateExpired,
                        FailureResultType::CertificateNotTrusted => Issue::CertificateNotTrusted,
                        FailureResultType::ValidationFailure => Issue::ValidationFailure,
                        FailureResultType::TlsaInvalid => Issue::TlsaInvalid,
                        FailureResultType::DnssecInvalid => Issue::DnssecInvalid,
                        FailureResultType::DaneRequired => Issue::DaneRequired,
                        FailureResultType::StsPolicyFetchError => Issue::StsPolicyFetchError,
                        FailureResultType::StsPolicyInvalid => Issue::StsPolicyInvalid,
                        FailureResultType::StsWebpkiInvalid => Issue::StsWebpkiInvalid,
                    });

                    // Add report type
                    details.types.insert(ReportType::Tls);
                }
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

fn detect_dmarc_issues(record: &RecordType, issues: &mut HashSet<Issue>) {
    if let Some(dkim) = &record.row.policy_evaluated.dkim {
        if *dkim != DmarcResultType::Pass {
            issues.insert(Issue::DkimPolicy);
        }
    }
    if let Some(spf) = &record.row.policy_evaluated.spf {
        if *spf != DmarcResultType::Pass {
            issues.insert(Issue::SpfPolicy);
        }
    }
    if let Some(dkim) = &record.auth_results.dkim {
        if dkim.iter().any(|x| x.result != DkimResultType::Pass) {
            issues.insert(Issue::DkimAuth);
        }
    }
    if record
        .auth_results
        .spf
        .iter()
        .any(|x| x.result != SpfResultType::Pass)
    {
        issues.insert(Issue::SpfAuth);
    }
}
