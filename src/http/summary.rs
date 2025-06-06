use crate::dmarc::{DkimResultType, DmarcResultType, SpfResultType};
use crate::state::{AppState, DmarcReportWithUid, TlsRptReportWithUid};
use crate::tlsrpt::{FailureResultType, PolicyType, TlsRptResultType};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize, Debug)]
pub struct SummaryFilters {
    /// Number of hours from current time backwards to include.
    /// Everything older will be excluded.
    /// None or a value of zero means the filter is disabled!
    time_span: Option<u64>,

    /// Domain to be filtered. Other domains will be ignored.
    /// None means the filter is disabled!
    domain: Option<String>,
}

impl SummaryFilters {
    fn url_decode(&self) -> Self {
        Self {
            time_span: self.time_span,
            domain: self
                .domain
                .as_ref()
                .and_then(|s| urlencoding::decode(s).ok())
                .map(|s| s.to_string()),
        }
    }
}

pub async fn handler(
    State(state): State<Arc<Mutex<AppState>>>,
    filters: Query<SummaryFilters>,
) -> impl IntoResponse {
    let filters = filters.url_decode();
    let guard = state.lock().await;
    let mut time_span = None;
    if let Some(hours) = filters.time_span {
        if hours > 0 {
            time_span = Some(Duration::hours(hours as i64));
        }
    }
    let summary = Summary::new(
        guard.mails.len(),
        Files {
            xml: guard.xml_files,
            json: guard.json_files,
        },
        Reports {
            dmarc: &guard.dmarc_reports,
            tlsrpt: &guard.tlsrpt_reports,
        },
        guard.last_update,
        time_span,
        filters.domain,
    );
    Json(summary)
}

#[derive(Serialize, Default, Clone)]
pub struct DmarcSummary {
    /// Number of XML files found in mails from IMAPinbox
    pub files: usize,

    /// Number of successfully parsed DMARC reports XML files found in IMAP inbox
    pub reports: usize,

    /// Map of organizations with number of corresponding DMARC reports
    pub orgs: HashMap<String, usize>,

    /// Map of domains with number of corresponding DMARC reports
    pub domains: HashMap<String, usize>,

    /// Map of DMARC SPF policy evaluation results
    pub spf_policy_results: HashMap<DmarcResultType, usize>,

    /// Map of DMARC DKIM policy evaluation results
    pub dkim_policy_results: HashMap<DmarcResultType, usize>,

    /// Map of DMARC SPF auth results
    pub spf_auth_results: HashMap<SpfResultType, usize>,

    /// Map of DMARC DKIM auth results
    pub dkim_auth_results: HashMap<DkimResultType, usize>,
}

#[derive(Serialize, Default, Clone)]
pub struct TlsRptSummary {
    /// Number of JSON files found in mails from IMAP inbox
    pub files: usize,

    /// Number of successfully parsed TLS-RPT reports JSON files found in IMAP inbox
    pub reports: usize,

    /// Map of organizations with number of corresponding TLS-RPT reports
    pub orgs: HashMap<String, usize>,

    /// Map of domains with number of corresponding TLS-RPT policy evaluations
    pub domains: HashMap<String, usize>,

    /// Map of TLS-RPT policy types
    pub policy_types: HashMap<PolicyType, usize>,

    /// Map of TLS-RPT STS policy evaluation results
    pub sts_policy_results: HashMap<TlsRptResultType, usize>,

    /// Map of TLS-RPT TLSA policy evaluation results
    pub tlsa_policy_results: HashMap<TlsRptResultType, usize>,

    /// Map of TLS-RPT STS failure results
    pub sts_failure_types: HashMap<FailureResultType, usize>,

    /// Map of TLS-RPT TLSA failure results
    pub tlsa_failure_types: HashMap<FailureResultType, usize>,
}

pub struct Files {
    /// Number of XML files found in mails from IMAP inbox
    pub xml: usize,

    /// Number of JSON files found in mails from IMAP inbox
    pub json: usize,
}

pub struct Reports<'a> {
    /// Parsed DMARC reports with mail UID and corresponding hash as key
    pub dmarc: &'a HashMap<String, DmarcReportWithUid>,

    /// Parsed TLS-RPT reports with mail UID and corresponding hash as key
    pub tlsrpt: &'a HashMap<String, TlsRptReportWithUid>,
}

#[derive(Serialize, Default, Clone)]
pub struct Summary {
    /// Number of mails from IMAP inbox
    pub mails: usize,

    /// Unix timestamp with time of last update
    pub last_update: u64,

    /// Information about DMARC reports
    pub dmarc: DmarcSummary,

    /// Information about TLS-RPT reports
    pub tlsrpt: TlsRptSummary,
}

impl Summary {
    pub fn new(
        mails: usize,
        files: Files,
        reports: Reports,
        last_update: u64,
        time_span: Option<Duration>,
        domain: Option<String>,
    ) -> Self {
        let dmarc_orgs: HashMap<String, usize> = HashMap::new();
        let dmarc_domains = HashMap::new();
        let spf_policy_results: HashMap<DmarcResultType, usize> = HashMap::new();
        let dkim_policy_results: HashMap<DmarcResultType, usize> = HashMap::new();
        let spf_auth_results: HashMap<SpfResultType, usize> = HashMap::new();
        let dkim_auth_results: HashMap<DkimResultType, usize> = HashMap::new();
        let mut dmarc = DmarcSummary {
            files: files.xml,
            reports: reports.dmarc.len(),
            orgs: dmarc_orgs,
            domains: dmarc_domains,
            spf_policy_results,
            dkim_policy_results,
            spf_auth_results,
            dkim_auth_results,
        };

        let tlsrpt_orgs: HashMap<String, usize> = HashMap::new();
        let tlsrpt_domains = HashMap::new();
        let policy_types: HashMap<PolicyType, usize> = HashMap::new();
        let sts_policy_results: HashMap<TlsRptResultType, usize> = HashMap::new();
        let tlsa_policy_results: HashMap<TlsRptResultType, usize> = HashMap::new();
        let sts_failure_types: HashMap<FailureResultType, usize> = HashMap::new();
        let tlsa_failure_types: HashMap<FailureResultType, usize> = HashMap::new();
        let mut tlsrpt = TlsRptSummary {
            files: files.json,
            reports: reports.tlsrpt.len(),
            orgs: tlsrpt_orgs,
            domains: tlsrpt_domains,
            policy_types,
            sts_policy_results,
            tlsa_policy_results,
            sts_failure_types,
            tlsa_failure_types,
        };

        let threshold = time_span.map(|d| (Utc::now() - d).timestamp() as u64);
        let threshold_datetime = time_span.map(|d| Utc::now() - d);
        for DmarcReportWithUid { report, .. } in reports.dmarc.values() {
            if let Some(threshold) = threshold {
                if report.report_metadata.date_range.end < threshold {
                    continue;
                }
            }
            if let Some(domain) = &domain {
                if report.policy_published.domain != *domain {
                    continue;
                }
            }
            let domain = report.policy_published.domain.clone();
            if let Some(entry) = dmarc.domains.get_mut(&domain) {
                *entry += 1;
            } else {
                dmarc.domains.insert(domain, 1);
            }
            let org = report.report_metadata.org_name.clone();
            if let Some(entry) = dmarc.orgs.get_mut(&org) {
                *entry += 1;
            } else {
                dmarc.orgs.insert(org, 1);
            }
            for record in &report.record {
                for r in &record.auth_results.spf {
                    if let Some(entry) = dmarc.spf_auth_results.get_mut(&r.result) {
                        *entry += record.row.count;
                    } else {
                        dmarc
                            .spf_auth_results
                            .insert(r.result.clone(), record.row.count);
                    }
                }
                if let Some(vec) = &record.auth_results.dkim {
                    for r in vec {
                        if let Some(entry) = dmarc.dkim_auth_results.get_mut(&r.result) {
                            *entry += record.row.count;
                        } else {
                            dmarc
                                .dkim_auth_results
                                .insert(r.result.clone(), record.row.count);
                        }
                    }
                }
                if let Some(result) = &record.row.policy_evaluated.spf {
                    if let Some(entry) = dmarc.spf_policy_results.get_mut(result) {
                        *entry += record.row.count;
                    } else {
                        dmarc
                            .spf_policy_results
                            .insert(result.clone(), record.row.count);
                    }
                }
                if let Some(result) = &record.row.policy_evaluated.dkim {
                    if let Some(entry) = dmarc.dkim_policy_results.get_mut(result) {
                        *entry += record.row.count;
                    } else {
                        dmarc
                            .dkim_policy_results
                            .insert(result.clone(), record.row.count);
                    }
                }
            }
        }
        for TlsRptReportWithUid { report, .. } in reports.tlsrpt.values() {
            if let Some(threshold_datetime) = threshold_datetime {
                if report.date_range.end_datetime < threshold_datetime {
                    continue;
                }
            }
            if let Some(domain) = &domain {
                if report
                    .policies
                    .iter()
                    .all(|p| p.policy.policy_domain != *domain)
                {
                    continue;
                }
            }
            let org = report.organization_name.clone();
            if let Some(entry) = tlsrpt.orgs.get_mut(&org) {
                *entry += 1;
            } else {
                tlsrpt.orgs.insert(org, 1);
            }
            for policy_result in report.policies.iter() {
                let domain = policy_result.policy.policy_domain.clone();
                if let Some(entry) = tlsrpt.domains.get_mut(&domain) {
                    *entry += 1;
                } else {
                    tlsrpt.domains.insert(domain, 1);
                }
                if let Some(entry) = tlsrpt
                    .policy_types
                    .get_mut(&policy_result.policy.policy_type)
                {
                    *entry += 1;
                } else {
                    tlsrpt
                        .policy_types
                        .insert(policy_result.policy.policy_type.clone(), 1);
                }
                let policy_results;
                let failure_types;
                match &policy_result.policy.policy_type {
                    PolicyType::Sts => {
                        policy_results = &mut tlsrpt.sts_policy_results;
                        failure_types = &mut tlsrpt.sts_failure_types;
                    }
                    PolicyType::Tlsa => {
                        policy_results = &mut tlsrpt.tlsa_policy_results;
                        failure_types = &mut tlsrpt.tlsa_failure_types;
                    }
                    PolicyType::NoPolicyFound => {
                        continue;
                    }
                }
                let success_count = policy_result.summary.total_successful_session_count;
                let failure_count = policy_result.summary.total_failure_session_count;
                if let Some(entry) = policy_results.get_mut(&TlsRptResultType::Successful) {
                    *entry += success_count;
                } else {
                    policy_results.insert(TlsRptResultType::Successful, success_count);
                }
                if let Some(entry) = policy_results.get_mut(&TlsRptResultType::Failure) {
                    *entry += failure_count;
                } else {
                    policy_results.insert(TlsRptResultType::Failure, failure_count);
                }
                if let Some(failure_details) = &policy_result.failure_details {
                    for failure_detail in failure_details {
                        if let Some(entry) = failure_types.get_mut(&failure_detail.result_type) {
                            *entry += failure_detail.failed_session_count;
                        } else {
                            failure_types.insert(
                                failure_detail.result_type.clone(),
                                failure_detail.failed_session_count,
                            );
                        }
                    }
                }
            }
        }
        Self {
            mails,
            last_update,
            dmarc,
            tlsrpt,
        }
    }
}
