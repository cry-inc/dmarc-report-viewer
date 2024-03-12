use crate::dmarc_report::{DkimResultType, DmarcResultType, Report, SpfResultType};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Default, Clone)]
pub struct Summary {
    /// Number of mails from IMAP inbox
    pub mails: usize,

    /// Number of XML files found in mails from IMAPinbox
    pub xml_files: usize,

    /// Number of successfully parsed DMARC reports XML files found in IMAP inbox
    pub reports: usize,

    /// Map of organizations with number of corresponding reports
    orgs: HashMap<String, usize>,

    /// Map of domains with number of corresponding reports
    domains: HashMap<String, usize>,

    /// Map of SPF policy evaluation results
    spf_policy_result: HashMap<PolicyResult, usize>,

    /// Map of DKIM policy evaluation results
    dkim_policy_result: HashMap<PolicyResult, usize>,

    /// All rows found in the reports
    rows: Vec<Row>,
}

impl Summary {
    pub fn new(mails: &[Vec<u8>], xml_files: &[Vec<u8>], reports: &[Report]) -> Self {
        let mut orgs: HashMap<String, usize> = HashMap::new();
        let mut domains = HashMap::new();
        let mut rows = Vec::new();
        let mut spf_policy_result: HashMap<PolicyResult, usize> = HashMap::new();
        let mut dkim_policy_result: HashMap<PolicyResult, usize> = HashMap::new();
        for report in reports {
            let org = report.report_metadata.org_name.clone();
            let domain = report.policy_published.domain.clone();
            for record in &report.record {
                let mut auth_dkim = Vec::new();
                if let Some(vec) = &record.auth_results.dkim {
                    for r in vec {
                        auth_dkim.push(DkimAuthResultStruct {
                            domain: domain.clone(),
                            selector: r.selector.clone(),
                            result: DkimAuthResult::from(&r.result),
                        });
                    }
                }
                let mut auth_spf = Vec::new();
                for r in &record.auth_results.spf {
                    auth_spf.push(SpfAuthResultStruct {
                        domain: domain.clone(),
                        result: SpfAuthResult::from(&r.result),
                    });
                }
                if let Some(result) = record.row.policy_evaluated.spf.as_ref() {
                    let result = PolicyResult::from(result);
                    if let Some(entry) = spf_policy_result.get_mut(&result) {
                        *entry += 1;
                    } else {
                        spf_policy_result.insert(result, 1);
                    }
                }
                if let Some(result) = record.row.policy_evaluated.dkim.as_ref() {
                    let result = PolicyResult::from(result);
                    if let Some(entry) = dkim_policy_result.get_mut(&result) {
                        *entry += 1;
                    } else {
                        dkim_policy_result.insert(result, 1);
                    }
                }
                let row = Row {
                    org: org.clone(),
                    domain: domain.clone(),
                    report_id: report.report_metadata.report_id.clone(),
                    source_ip: record.row.source_ip.to_string(),
                    count: record.row.count,
                    dkim_policy: record
                        .row
                        .policy_evaluated
                        .dkim
                        .as_ref()
                        .map(PolicyResult::from),
                    spf_policy: record
                        .row
                        .policy_evaluated
                        .spf
                        .as_ref()
                        .map(PolicyResult::from),
                    auth_dkim,
                    auth_spf,
                };
                rows.push(row);
            }
            if let Some(entry) = orgs.get_mut(&org) {
                *entry += 1;
            } else {
                orgs.insert(org, 1);
            }
            if let Some(entry) = domains.get_mut(&domain) {
                *entry += 1;
            } else {
                domains.insert(domain, 1);
            }
        }
        Self {
            mails: mails.len(),
            reports: reports.len(),
            xml_files: xml_files.len(),
            orgs,
            domains,
            spf_policy_result,
            dkim_policy_result,
            rows,
        }
    }
}

#[derive(Serialize, Clone, Hash, PartialEq, Eq)]
enum PolicyResult {
    Pass,
    Fail,
}

impl PolicyResult {
    fn from(result: &DmarcResultType) -> Self {
        match result {
            DmarcResultType::Pass => Self::Pass,
            DmarcResultType::Fail => Self::Fail,
        }
    }
}

#[derive(Serialize, Clone)]
enum DkimAuthResult {
    None,
    Pass,
    Fail,
    Policy,
    Neutral,
    TemporaryError,
    PermanentError,
}

impl DkimAuthResult {
    fn from(result: &DkimResultType) -> Self {
        match result {
            DkimResultType::None => Self::None,
            DkimResultType::Pass => Self::Pass,
            DkimResultType::Fail => Self::Fail,
            DkimResultType::Policy => Self::Policy,
            DkimResultType::Neutral => Self::Neutral,
            DkimResultType::TemporaryError => Self::TemporaryError,
            DkimResultType::PermanentError => Self::PermanentError,
        }
    }
}

#[derive(Serialize, Clone)]
enum SpfAuthResult {
    None,
    Neutral,
    Pass,
    Fail,
    SoftFail,
    TemporaryError,
    PermanentError,
}

impl SpfAuthResult {
    fn from(result: &SpfResultType) -> Self {
        match result {
            SpfResultType::None => SpfAuthResult::None,
            SpfResultType::Neutral => SpfAuthResult::Neutral,
            SpfResultType::Pass => SpfAuthResult::Pass,
            SpfResultType::Fail => SpfAuthResult::Fail,
            SpfResultType::SoftFail => SpfAuthResult::SoftFail,
            SpfResultType::TemporaryError => SpfAuthResult::TemporaryError,
            SpfResultType::PermanentError => SpfAuthResult::PermanentError,
        }
    }
}

#[derive(Serialize, Clone)]
struct SpfAuthResultStruct {
    domain: String,
    result: SpfAuthResult,
}

#[derive(Serialize, Clone)]
struct DkimAuthResultStruct {
    domain: String,
    selector: Option<String>,
    result: DkimAuthResult,
}

#[derive(Serialize, Clone)]
struct Row {
    org: String,
    domain: String,
    report_id: String,
    source_ip: String,
    count: u32,
    dkim_policy: Option<PolicyResult>,
    spf_policy: Option<PolicyResult>,
    auth_dkim: Vec<DkimAuthResultStruct>,
    auth_spf: Vec<SpfAuthResultStruct>,
}
