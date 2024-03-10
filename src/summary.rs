use crate::dmarc_report::{feedback, DKIMResultType, DMARCResultType, SPFResultType};
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

    /// All rows found in the reports
    rows: Vec<Row>,
}

impl Summary {
    pub fn new(mails: &[Vec<u8>], xml_files: &[Vec<u8>], reports: &[feedback]) -> Self {
        let mut orgs: HashMap<String, usize> = HashMap::new();
        let mut domains = HashMap::new();
        let mut rows = Vec::new();
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
            rows,
        }
    }
}

#[derive(Serialize, Clone)]
enum PolicyResult {
    Pass,
    Fail,
}

impl PolicyResult {
    fn from(result: &DMARCResultType) -> Self {
        match result {
            DMARCResultType::pass => Self::Pass,
            DMARCResultType::fail => Self::Fail,
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
    TempError,
    PermError,
}

impl DkimAuthResult {
    fn from(result: &DKIMResultType) -> Self {
        match result {
            DKIMResultType::none => Self::None,
            DKIMResultType::pass => Self::Pass,
            DKIMResultType::fail => Self::Fail,
            DKIMResultType::policy => Self::Policy,
            DKIMResultType::neutral => Self::Neutral,
            DKIMResultType::temperror => Self::TempError,
            DKIMResultType::permerror => Self::PermError,
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
    TempError,
    PermError,
}

impl SpfAuthResult {
    fn from(result: &SPFResultType) -> Self {
        match result {
            SPFResultType::none => SpfAuthResult::None,
            SPFResultType::neutral => SpfAuthResult::Neutral,
            SPFResultType::pass => SpfAuthResult::Pass,
            SPFResultType::fail => SpfAuthResult::Fail,
            SPFResultType::softfail => SpfAuthResult::SoftFail,
            SPFResultType::temperror => SpfAuthResult::TempError,
            SPFResultType::permerror => SpfAuthResult::PermError,
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
