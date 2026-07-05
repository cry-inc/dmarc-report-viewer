use crate::dmarc::{DkimResultType, DmarcResultType, SpfResultType};
use crate::state::{AppState, DmarcReportWithMailId, TlsReportWithMailId};
use crate::tls::{FailureResultType, PolicyType, TlsResultType};
use axum::Json;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
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
    fn url_decode(&mut self) {
        self.domain = self
            .domain
            .as_ref()
            .and_then(|s| urlencoding::decode(s).ok())
            .map(|s| s.to_string());
    }
}

pub async fn handler(
    State(state): State<Arc<Mutex<AppState>>>,
    mut filters: Query<SummaryFilters>,
) -> impl IntoResponse {
    filters.url_decode();
    let guard = state.lock().await;
    let mut time_span = None;
    if let Some(hours) = filters.time_span
        && hours > 0
    {
        time_span = Some(Duration::hours(hours as i64));
    }
    let summary = Summary::new(
        guard.mails.len(),
        Files {
            xml: guard.xml_files,
            json: guard.json_files,
        },
        Reports {
            dmarc: &guard.dmarc_reports,
            tls: &guard.tls_reports,
        },
        guard.last_update,
        time_span,
        filters.domain.clone(),
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
pub struct TlsSummary {
    /// Number of JSON files found in mails from IMAP inbox
    pub files: usize,

    /// Number of successfully parsed SMTP TLS reports JSON files found in IMAP inbox
    pub reports: usize,

    /// Map of organizations with number of corresponding SMTP TLS reports
    pub orgs: HashMap<String, usize>,

    /// Map of domains with number of corresponding SMTP TLS policy evaluations
    pub domains: HashMap<String, usize>,

    /// Map of SMTP TLS policy types
    pub policy_types: HashMap<PolicyType, usize>,

    /// Map of SMTP TLS STS policy evaluation results
    pub sts_policy_results: HashMap<TlsResultType, usize>,

    /// Map of SMTP TLS TLSA policy evaluation results
    pub tlsa_policy_results: HashMap<TlsResultType, usize>,

    /// Map of SMTP TLS STS failure results
    pub sts_failure_types: HashMap<FailureResultType, usize>,

    /// Map of SMTP TLS TLSA failure results
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
    pub dmarc: &'a BTreeMap<String, DmarcReportWithMailId>,

    /// Parsed SMTP TLS reports with mail UID and corresponding hash as key
    pub tls: &'a BTreeMap<String, TlsReportWithMailId>,
}

#[derive(Serialize, Default, Clone)]
pub struct Summary {
    /// Number of mails from IMAP inbox
    pub mails: usize,

    /// Unix timestamp with time of last update
    pub last_update: u64,

    /// Information about DMARC reports
    pub dmarc: DmarcSummary,

    /// Information about SMTP TLS reports
    pub tls: TlsSummary,
}

impl Summary {
    pub fn new(
        mails: usize,
        files: Files,
        reports: Reports,
        last_update: u64,
        time_span: Option<Duration>,
        domain_filter: Option<String>,
    ) -> Self {
        let mut dmarc = DmarcSummary {
            files: files.xml,
            reports: reports.dmarc.len(),
            ..Default::default()
        };

        let mut tls = TlsSummary {
            files: files.json,
            reports: reports.tls.len(),
            ..Default::default()
        };

        let threshold = time_span.map(|d| (Utc::now() - d).timestamp() as u64);
        let threshold_datetime = time_span.map(|d| Utc::now() - d);
        let domain_filter = domain_filter.map(|d| d.to_lowercase());
        for DmarcReportWithMailId { report, .. } in reports.dmarc.values() {
            if let Some(threshold) = threshold
                && report.report_metadata.date_range.end < threshold
            {
                continue;
            }
            if let Some(df) = &domain_filter
                && report.policy_published.domain.to_lowercase() != *df
            {
                continue;
            }
            let domain = report.policy_published.domain.to_lowercase();
            *dmarc.domains.entry(domain).or_insert(0) += 1;
            let org = report.report_metadata.org_name.clone();
            *dmarc.orgs.entry(org).or_insert(0) += 1;
            for record in &report.record {
                for r in &record.auth_results.spf {
                    *dmarc.spf_auth_results.entry(r.result.clone()).or_insert(0) +=
                        record.row.count;
                }
                if let Some(vec) = &record.auth_results.dkim {
                    for r in vec {
                        *dmarc.dkim_auth_results.entry(r.result.clone()).or_insert(0) +=
                            record.row.count;
                    }
                }
                if let Some(result) = &record.row.policy_evaluated.spf {
                    *dmarc.spf_policy_results.entry(result.clone()).or_insert(0) +=
                        record.row.count;
                }
                if let Some(result) = &record.row.policy_evaluated.dkim {
                    *dmarc.dkim_policy_results.entry(result.clone()).or_insert(0) +=
                        record.row.count;
                }
            }
        }
        for TlsReportWithMailId { report, .. } in reports.tls.values() {
            if let Some(threshold_datetime) = threshold_datetime
                && report.date_range.end_datetime < threshold_datetime
            {
                continue;
            }
            if let Some(df) = &domain_filter
                && report
                    .policies
                    .iter()
                    .all(|p| p.policy.policy_domain.to_lowercase() != *df)
            {
                continue;
            }
            let org = report.organization_name.clone();
            *tls.orgs.entry(org).or_insert(0) += 1;
            for policy_result in report.policies.iter() {
                let domain = policy_result.policy.policy_domain.to_lowercase();
                *tls.domains.entry(domain).or_insert(0) += 1;
                *tls.policy_types
                    .entry(policy_result.policy.policy_type.clone())
                    .or_insert(0) += 1;
                let (policy_results, failure_types) = match &policy_result.policy.policy_type {
                    PolicyType::Sts => (&mut tls.sts_policy_results, &mut tls.sts_failure_types),
                    PolicyType::Tlsa => (&mut tls.tlsa_policy_results, &mut tls.tlsa_failure_types),
                    PolicyType::NoPolicyFound => {
                        continue;
                    }
                };
                let success_count = policy_result.summary.total_successful_session_count;
                let failure_count = policy_result.summary.total_failure_session_count;
                *policy_results.entry(TlsResultType::Successful).or_insert(0) += success_count;
                *policy_results.entry(TlsResultType::Failure).or_insert(0) += failure_count;
                if let Some(failure_details) = &policy_result.failure_details {
                    for failure_detail in failure_details {
                        *failure_types
                            .entry(failure_detail.result_type.clone())
                            .or_insert(0) += failure_detail.failed_session_count;
                    }
                }
            }
        }
        Self {
            mails,
            last_update,
            dmarc,
            tls,
        }
    }
}
