// Original code from https://github.com/bbustin/dmarc_aggregate_parser/
// Its based upon appendix C of the DMARC RFC: https://tools.ietf.org/html/rfc7489#appendix-C

use serde::Deserialize;
use std::net::IpAddr;

#[derive(Deserialize)]
pub struct DateRangeType {
    pub begin: u32,
    pub end: u32,
}

#[derive(Deserialize)]
pub struct ReportMetadataType {
    pub org_name: String,
    pub email: String,
    pub extra_contact_info: Option<String>,
    pub report_id: String,
    pub date_range: DateRangeType,
    pub error: Option<Vec<String>>,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
pub enum AlignmentType {
    /// Relaxed
    r,
    /// Strict
    s,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
pub enum DispositionType {
    /// There is no preference on how a failed DMARC should be handled.
    none,
    /// The message should be quarantined. This usually means it will be placed in the `spam` folder
    /// of the user
    quarantine,
    /// The message should be regjected.
    reject,
}

#[derive(Deserialize)]
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
#[derive(Deserialize)]
pub enum DMARCResultType {
    pass,
    fail,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
pub enum PolicyOverrideType {
    forwarded,
    sampled_out,
    trusted_forwarder,
    mailing_list,
    local_policy,
    other,
}

#[derive(Deserialize)]
pub struct PolicyOverrideReason {
    pub r#type: PolicyOverrideType,
    pub comment: Option<String>,
}

#[derive(Deserialize)]
pub struct PolicyEvaluatedType {
    pub disposition: DispositionType,
    pub dkim: Option<DMARCResultType>,
    pub spf: Option<DMARCResultType>,
    pub reason: Option<Vec<PolicyOverrideReason>>,
}

#[derive(Deserialize)]
pub struct RowType {
    pub source_ip: IpAddr,
    pub count: u32,
    pub policy_evaluated: PolicyEvaluatedType,
}

#[derive(Deserialize)]
pub struct IdentifierType {
    pub envelope_to: Option<String>,
    pub envelope_from: Option<String>,
    pub header_from: String,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
pub enum DKIMResultType {
    none,
    pass,
    fail,
    policy,
    neutral,
    temperror,
    permerror,
}

#[derive(Deserialize)]
pub struct DKIMAuthResultType {
    pub domain: String,
    pub selector: Option<String>,
    pub result: DKIMResultType,
    pub human_result: Option<String>,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
pub enum SPFDomainScope {
    helo,
    mfrom,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
pub enum SPFResultType {
    none,
    neutral,
    pass,
    fail,
    softfail,
    temperror,
    permerror,
}

#[derive(Deserialize)]
pub struct SPFAuthResultType {
    pub domain: String,
    pub scope: Option<SPFDomainScope>,
    pub result: SPFResultType,
}

#[derive(Deserialize)]
pub struct AuthResultType {
    pub dkim: Option<Vec<DKIMAuthResultType>>,
    pub spf: Vec<SPFAuthResultType>,
}

#[derive(Deserialize)]
pub struct RecordType {
    pub row: RowType,
    pub identifiers: IdentifierType,
    pub auth_results: AuthResultType,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
pub struct feedback {
    pub version: Option<String>,
    pub report_metadata: ReportMetadataType,
    pub policy_published: PolicyPublishedType,
    pub record: Vec<RecordType>,
}
