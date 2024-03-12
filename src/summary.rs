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
    spf_policy_results: HashMap<PolicyResult, usize>,

    /// Map of DKIM policy evaluation results
    dkim_policy_results: HashMap<PolicyResult, usize>,

    /// Map of SPF auth results
    spf_auth_results: HashMap<SpfAuthResult, usize>,

    /// Map of DKIM auth results
    dkim_auth_results: HashMap<DkimAuthResult, usize>,
}

impl Summary {
    pub fn new(mails: &[Vec<u8>], xml_files: &[Vec<u8>], reports: &[Report]) -> Self {
        let mut orgs: HashMap<String, usize> = HashMap::new();
        let mut domains = HashMap::new();
        let mut spf_policy_results: HashMap<PolicyResult, usize> = HashMap::new();
        let mut dkim_policy_results: HashMap<PolicyResult, usize> = HashMap::new();
        let mut spf_auth_results: HashMap<SpfAuthResult, usize> = HashMap::new();
        let mut dkim_auth_results: HashMap<DkimAuthResult, usize> = HashMap::new();
        for report in reports {
            for record in &report.record {
                for r in &record.auth_results.spf {
                    let result = SpfAuthResult::from(&r.result);
                    if let Some(entry) = spf_auth_results.get_mut(&result) {
                        *entry += 1;
                    } else {
                        spf_auth_results.insert(result, 1);
                    }
                }
                if let Some(vec) = &record.auth_results.dkim {
                    for r in vec {
                        let result = DkimAuthResult::from(&r.result);
                        if let Some(entry) = dkim_auth_results.get_mut(&result) {
                            *entry += 1;
                        } else {
                            dkim_auth_results.insert(result, 1);
                        }
                    }
                }
                if let Some(result) = record.row.policy_evaluated.spf.as_ref() {
                    let result = PolicyResult::from(result);
                    if let Some(entry) = spf_policy_results.get_mut(&result) {
                        *entry += 1;
                    } else {
                        spf_policy_results.insert(result, 1);
                    }
                }
                if let Some(result) = record.row.policy_evaluated.dkim.as_ref() {
                    let result = PolicyResult::from(result);
                    if let Some(entry) = dkim_policy_results.get_mut(&result) {
                        *entry += 1;
                    } else {
                        dkim_policy_results.insert(result, 1);
                    }
                }
            }
            let org = report.report_metadata.org_name.clone();
            if let Some(entry) = orgs.get_mut(&org) {
                *entry += 1;
            } else {
                orgs.insert(org, 1);
            }
            let domain = report.policy_published.domain.clone();
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
            spf_policy_results,
            dkim_policy_results,
            spf_auth_results,
            dkim_auth_results,
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

#[derive(Serialize, Clone, Hash, PartialEq, Eq)]
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

#[derive(Serialize, Clone, Hash, PartialEq, Eq)]
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
