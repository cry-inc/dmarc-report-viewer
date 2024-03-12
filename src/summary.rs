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
    spf_policy_results: HashMap<DmarcResultType, usize>,

    /// Map of DKIM policy evaluation results
    dkim_policy_results: HashMap<DmarcResultType, usize>,

    /// Map of SPF auth results
    spf_auth_results: HashMap<SpfResultType, usize>,

    /// Map of DKIM auth results
    dkim_auth_results: HashMap<DkimResultType, usize>,
}

impl Summary {
    pub fn new(mails: &[Vec<u8>], xml_files: &[Vec<u8>], reports: &[Report]) -> Self {
        let mut orgs: HashMap<String, usize> = HashMap::new();
        let mut domains = HashMap::new();
        let mut spf_policy_results: HashMap<DmarcResultType, usize> = HashMap::new();
        let mut dkim_policy_results: HashMap<DmarcResultType, usize> = HashMap::new();
        let mut spf_auth_results: HashMap<SpfResultType, usize> = HashMap::new();
        let mut dkim_auth_results: HashMap<DkimResultType, usize> = HashMap::new();
        for report in reports {
            for record in &report.record {
                for r in &record.auth_results.spf {
                    if let Some(entry) = spf_auth_results.get_mut(&r.result) {
                        *entry += 1;
                    } else {
                        spf_auth_results.insert(r.result.clone(), 1);
                    }
                }
                if let Some(vec) = &record.auth_results.dkim {
                    for r in vec {
                        if let Some(entry) = dkim_auth_results.get_mut(&r.result) {
                            *entry += 1;
                        } else {
                            dkim_auth_results.insert(r.result.clone(), 1);
                        }
                    }
                }
                if let Some(result) = &record.row.policy_evaluated.spf {
                    if let Some(entry) = spf_policy_results.get_mut(result) {
                        *entry += 1;
                    } else {
                        spf_policy_results.insert(result.clone(), 1);
                    }
                }
                if let Some(result) = &record.row.policy_evaluated.dkim {
                    if let Some(entry) = dkim_policy_results.get_mut(result) {
                        *entry += 1;
                    } else {
                        dkim_policy_results.insert(result.clone(), 1);
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
