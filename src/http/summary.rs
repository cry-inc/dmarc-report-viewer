use crate::dmarc::{DkimResultType, DmarcResultType, SpfResultType};
use crate::state::{AppState, DmarcReportWithUid};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handler(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let guard = state.lock().await;
    let summary = Summary::new(
        guard.mails.len(),
        guard.xml_files,
        &guard.dmarc_reports,
        guard.last_update,
    );
    Json(summary)
}

#[derive(Serialize, Default, Clone)]
pub struct Summary {
    /// Number of mails from IMAP inbox
    pub mails: usize,

    /// Number of XML files found in mails from IMAPinbox
    pub xml_files: usize,

    /// Number of successfully parsed DMARC reports XML files found in IMAP inbox
    pub dmarc_reports: usize,

    /// Unix timestamp with time of last update
    pub last_update: u64,

    /// Map of organizations with number of corresponding reports
    pub dmarc_orgs: HashMap<String, usize>,

    /// Map of domains with number of corresponding reports
    pub dmarc_domains: HashMap<String, usize>,

    /// Map of DMARC SPF policy evaluation results
    pub spf_policy_results: HashMap<DmarcResultType, usize>,

    /// Map of DMARC DKIM policy evaluation results
    pub dkim_policy_results: HashMap<DmarcResultType, usize>,

    /// Map of DMARC SPF auth results
    pub spf_auth_results: HashMap<SpfResultType, usize>,

    /// Map of DMARC DKIM auth results
    pub dkim_auth_results: HashMap<DkimResultType, usize>,
}

impl Summary {
    pub fn new(
        mails: usize,
        xml_files: usize,
        dmarc_reports: &HashMap<String, DmarcReportWithUid>,
        last_update: u64,
    ) -> Self {
        let mut dmarc_orgs: HashMap<String, usize> = HashMap::new();
        let mut dmarc_domains = HashMap::new();
        let mut spf_policy_results: HashMap<DmarcResultType, usize> = HashMap::new();
        let mut dkim_policy_results: HashMap<DmarcResultType, usize> = HashMap::new();
        let mut spf_auth_results: HashMap<SpfResultType, usize> = HashMap::new();
        let mut dkim_auth_results: HashMap<DkimResultType, usize> = HashMap::new();
        for DmarcReportWithUid { report, .. } in dmarc_reports.values() {
            for record in &report.record {
                for r in &record.auth_results.spf {
                    if let Some(entry) = spf_auth_results.get_mut(&r.result) {
                        *entry += record.row.count;
                    } else {
                        spf_auth_results.insert(r.result.clone(), record.row.count);
                    }
                }
                if let Some(vec) = &record.auth_results.dkim {
                    for r in vec {
                        if let Some(entry) = dkim_auth_results.get_mut(&r.result) {
                            *entry += record.row.count;
                        } else {
                            dkim_auth_results.insert(r.result.clone(), record.row.count);
                        }
                    }
                }
                if let Some(result) = &record.row.policy_evaluated.spf {
                    if let Some(entry) = spf_policy_results.get_mut(result) {
                        *entry += record.row.count;
                    } else {
                        spf_policy_results.insert(result.clone(), record.row.count);
                    }
                }
                if let Some(result) = &record.row.policy_evaluated.dkim {
                    if let Some(entry) = dkim_policy_results.get_mut(result) {
                        *entry += record.row.count;
                    } else {
                        dkim_policy_results.insert(result.clone(), record.row.count);
                    }
                }
            }
            let org = report.report_metadata.org_name.clone();
            if let Some(entry) = dmarc_orgs.get_mut(&org) {
                *entry += 1;
            } else {
                dmarc_orgs.insert(org, 1);
            }
            let domain = report.policy_published.domain.clone();
            if let Some(entry) = dmarc_domains.get_mut(&domain) {
                *entry += 1;
            } else {
                dmarc_domains.insert(domain, 1);
            }
        }
        Self {
            mails,
            xml_files,
            last_update,
            dmarc_reports: dmarc_reports.len(),
            dmarc_orgs,
            dmarc_domains,
            spf_policy_results,
            dkim_policy_results,
            spf_auth_results,
            dkim_auth_results,
        }
    }
}
