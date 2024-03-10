// Original code from https://github.com/bbustin/dmarc_aggregate_parser/
// Its based upon appendix C of the DMARC RFC: https://tools.ietf.org/html/rfc7489#appendix-C

use serde::Deserialize;
use std::net::IpAddr;

#[derive(Debug, Deserialize)]
pub struct DateRangeType {
    pub begin: u32,
    pub end: u32,
}

#[derive(Debug, Deserialize)]
pub struct ReportMetadataType {
    pub org_name: String,
    pub email: String,
    pub extra_contact_info: Option<String>,
    pub report_id: String,
    pub date_range: DateRangeType,
    pub error: Option<Vec<String>>,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, PartialEq)]
pub enum AlignmentType {
    /// Relaxed
    r,
    /// Strict
    s,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, PartialEq)]
pub enum DispositionType {
    /// There is no preference on how a failed DMARC should be handled.
    none,
    /// The message should be quarantined. This usually means it will be placed in the `spam` folder
    /// of the user
    quarantine,
    /// The message should be regjected.
    reject,
}

#[derive(Debug, Deserialize)]
pub struct PolicyPublishedType {
    pub domain: String,
    pub adkim: Option<AlignmentType>,
    pub aspf: Option<AlignmentType>,
    pub p: DispositionType,
    pub sp: Option<DispositionType>,
    pub pct: u8,
    pub fo: Option<String>,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, PartialEq)]
pub enum DMARCResultType {
    pass,
    fail,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub enum PolicyOverrideType {
    forwarded,
    sampled_out,
    trusted_forwarder,
    mailing_list,
    local_policy,
    other,
}

#[derive(Debug, Deserialize)]
pub struct PolicyOverrideReason {
    pub r#type: PolicyOverrideType,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PolicyEvaluatedType {
    pub disposition: DispositionType,
    pub dkim: Option<DMARCResultType>,
    pub spf: Option<DMARCResultType>,
    pub reason: Option<Vec<PolicyOverrideReason>>,
}

#[derive(Debug, Deserialize)]
pub struct RowType {
    pub source_ip: IpAddr,
    pub count: u32,
    pub policy_evaluated: PolicyEvaluatedType,
}

#[derive(Debug, Deserialize)]
pub struct IdentifierType {
    pub envelope_to: Option<String>,
    pub envelope_from: Option<String>,
    pub header_from: String,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, PartialEq)]
pub enum DKIMResultType {
    none,
    pass,
    fail,
    policy,
    neutral,
    temperror,
    permerror,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct DKIMAuthResultType {
    pub domain: String,
    pub selector: Option<String>,
    pub result: DKIMResultType,
    pub human_result: Option<String>,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, PartialEq)]
pub enum SPFDomainScope {
    helo,
    mfrom,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize, PartialEq)]
pub enum SPFResultType {
    none,
    neutral,
    pass,
    fail,
    softfail,
    temperror,
    permerror,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SPFAuthResultType {
    pub domain: String,
    pub scope: Option<SPFDomainScope>,
    pub result: SPFResultType,
}

#[derive(Debug, Deserialize)]
pub struct AuthResultType {
    pub dkim: Option<Vec<DKIMAuthResultType>>,
    pub spf: Vec<SPFAuthResultType>,
}

#[derive(Debug, Deserialize)]
pub struct RecordType {
    pub row: RowType,
    pub identifiers: IdentifierType,
    pub auth_results: AuthResultType,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub struct feedback {
    pub version: Option<String>,
    pub report_metadata: ReportMetadataType,
    pub policy_published: PolicyPublishedType,
    pub record: Vec<RecordType>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_aol_report() {
        let reader = File::open("testdata/dmarc-reports/aol.xml").unwrap();
        let report: feedback = serde_xml_rs::from_reader(reader).unwrap();

        // Check metadata
        assert_eq!(report.report_metadata.org_name, "AOL");
        assert_eq!(report.report_metadata.email, "postmaster@aol.com");
        assert_eq!(report.report_metadata.report_id, "website.com_1504828800");
        assert_eq!(report.report_metadata.date_range.begin, 1504742400);
        assert_eq!(report.report_metadata.date_range.end, 1504828800);

        // Check policy
        assert_eq!(report.policy_published.domain, "website.com");
        assert_eq!(report.policy_published.adkim, Some(AlignmentType::r));
        assert_eq!(report.policy_published.aspf, Some(AlignmentType::r));
        assert_eq!(report.policy_published.p, DispositionType::reject);
        assert_eq!(report.policy_published.sp, Some(DispositionType::reject));
        assert_eq!(report.policy_published.pct, 100);

        // Check record
        assert_eq!(report.record.len(), 1);
        let record = report.record.first().unwrap();
        assert_eq!(record.row.source_ip.to_string(), "125.125.125.125");
        assert_eq!(record.row.count, 1);
        assert_eq!(
            record.row.policy_evaluated.disposition,
            DispositionType::none
        );
        assert_eq!(
            record.row.policy_evaluated.dkim,
            Some(DMARCResultType::pass)
        );
        assert_eq!(record.row.policy_evaluated.spf, Some(DMARCResultType::pass));
        assert_eq!(record.identifiers.header_from, "website.com");
        assert_eq!(
            record.auth_results.dkim,
            Some(vec![DKIMAuthResultType {
                domain: String::from("website.com"),
                selector: None,
                result: DKIMResultType::pass,
                human_result: None
            }])
        );
        assert_eq!(
            record.auth_results.spf,
            vec![SPFAuthResultType {
                domain: String::from("website.com"),
                scope: Some(SPFDomainScope::mfrom),
                result: SPFResultType::pass,
            }]
        );
    }
}
