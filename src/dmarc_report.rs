// Original code before customization was copied from:
// https://github.com/bbustin/dmarc_aggregate_parser/
// Its based upon appendix C of the DMARC RFC:
// https://tools.ietf.org/html/rfc7489#appendix-C

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

#[derive(Debug, Deserialize, PartialEq)]
pub enum AlignmentType {
    #[serde(rename = "r")]
    Relaxed,
    #[serde(rename = "s")]
    Strict,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DispositionType {
    /// There is no preference on how a failed DMARC should be handled.
    None,
    /// The message should be quarantined. This usually means it will be placed in the `spam` folder of the user.
    Quarantine,
    /// The message should be rejected.
    Reject,
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

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DmarcResultType {
    Pass,
    Fail,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyOverrideType {
    Forwarded,
    SampledOut,
    TrustedForwarder,
    MailingList,
    LocalPolicy,
    Other,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct PolicyOverrideReason {
    #[serde(rename = "type")]
    pub kind: PolicyOverrideType,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PolicyEvaluatedType {
    pub disposition: DispositionType,
    pub dkim: Option<DmarcResultType>,
    pub spf: Option<DmarcResultType>,
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

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DkimResultType {
    None,
    Pass,
    Fail,
    Policy,
    Neutral,
    #[serde(rename = "temperror")]
    TemporaryError,
    #[serde(rename = "permerror")]
    PermanentError,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct DkimAuthResultType {
    pub domain: String,
    pub selector: Option<String>,
    pub result: DkimResultType,
    pub human_result: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SpfDomainScope {
    Helo,
    #[serde(rename = "mfrom")]
    MailForm,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SpfResultType {
    None,
    Neutral,
    Pass,
    Fail,
    #[serde(rename = "softfail")]
    SoftFail,
    #[serde(rename = "temperror")]
    TemporaryError,
    #[serde(rename = "permerror")]
    PermanentError,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SpfAuthResultType {
    pub domain: String,
    pub scope: Option<SpfDomainScope>,
    pub result: SpfResultType,
}

#[derive(Debug, Deserialize)]
pub struct AuthResultType {
    pub dkim: Option<Vec<DkimAuthResultType>>,
    pub spf: Vec<SpfAuthResultType>,
}

#[derive(Debug, Deserialize)]
pub struct RecordType {
    pub row: RowType,
    pub identifiers: IdentifierType,
    pub auth_results: AuthResultType,
}

#[derive(Debug, Deserialize)]
pub struct Report {
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
    fn aol_report() {
        let reader = File::open("testdata/dmarc-reports/aol.xml").unwrap();
        let report: Report = serde_xml_rs::from_reader(reader).unwrap();

        // Check metadata
        assert_eq!(report.report_metadata.org_name, "AOL");
        assert_eq!(report.report_metadata.email, "postmaster@aol.com");
        assert_eq!(report.report_metadata.report_id, "website.com_1504828800");
        assert_eq!(report.report_metadata.date_range.begin, 1504742400);
        assert_eq!(report.report_metadata.date_range.end, 1504828800);

        // Check policy
        assert_eq!(report.policy_published.domain, "website.com");
        assert_eq!(report.policy_published.adkim, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.aspf, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.p, DispositionType::Reject);
        assert_eq!(report.policy_published.sp, Some(DispositionType::Reject));
        assert_eq!(report.policy_published.pct, 100);

        // Check record
        assert_eq!(report.record.len(), 1);
        let record = report.record.first().unwrap();
        assert_eq!(record.row.source_ip.to_string(), "125.125.125.125");
        assert_eq!(record.row.count, 1);
        assert_eq!(
            record.row.policy_evaluated.disposition,
            DispositionType::None
        );
        assert_eq!(
            record.row.policy_evaluated.dkim,
            Some(DmarcResultType::Pass)
        );
        assert_eq!(record.row.policy_evaluated.spf, Some(DmarcResultType::Pass));
        assert_eq!(record.identifiers.header_from, "website.com");
        assert_eq!(
            record.auth_results.dkim,
            Some(vec![DkimAuthResultType {
                domain: String::from("website.com"),
                selector: None,
                result: DkimResultType::Pass,
                human_result: None
            }])
        );
        assert_eq!(
            record.auth_results.spf,
            vec![SpfAuthResultType {
                domain: String::from("website.com"),
                scope: Some(SpfDomainScope::MailForm),
                result: SpfResultType::Pass,
            }]
        );
    }

    #[test]
    fn acme_report() {
        let reader = File::open("testdata/dmarc-reports/acme.xml").unwrap();
        let report: Report = serde_xml_rs::from_reader(reader).unwrap();

        // Check metadata
        assert_eq!(report.report_metadata.org_name, "acme.com");
        assert_eq!(
            report.report_metadata.email,
            "noreply-dmarc-support@acme.com"
        );
        assert_eq!(
            report.report_metadata.extra_contact_info.as_deref(),
            Some("http://acme.com/dmarc/support")
        );
        assert_eq!(report.report_metadata.report_id, "9391651994964116463");
        assert_eq!(
            report.report_metadata.error,
            Some(vec![String::from("There was a sample error.")])
        );
        assert_eq!(report.report_metadata.date_range.begin, 1335571200);
        assert_eq!(report.report_metadata.date_range.end, 1335657599);

        // Check policy
        assert_eq!(report.policy_published.domain, "example.com");
        assert_eq!(report.policy_published.adkim, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.aspf, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.p, DispositionType::None);
        assert_eq!(report.policy_published.sp, Some(DispositionType::None));
        assert_eq!(report.policy_published.pct, 100);
        assert_eq!(report.policy_published.fo, Some(String::from("1")));

        // Check record
        assert_eq!(report.record.len(), 1);
        let record = report.record.first().unwrap();
        assert_eq!(record.row.source_ip.to_string(), "72.150.241.94");
        assert_eq!(record.row.count, 2);
        assert_eq!(
            record.row.policy_evaluated.disposition,
            DispositionType::None
        );
        assert_eq!(
            record.row.policy_evaluated.dkim,
            Some(DmarcResultType::Fail)
        );
        assert_eq!(record.row.policy_evaluated.spf, Some(DmarcResultType::Pass));
        assert_eq!(
            record.row.policy_evaluated.reason,
            Some(vec![PolicyOverrideReason {
                kind: PolicyOverrideType::Other,
                comment: Some(String::from(
                    "DMARC Policy overridden for incoherent example."
                ))
            }])
        );
        assert_eq!(record.identifiers.header_from, "example.com");
        assert_eq!(
            record.identifiers.envelope_from,
            Some(String::from("example.com"))
        );
        assert_eq!(
            record.identifiers.envelope_to,
            Some(String::from("acme.com"))
        );
        assert_eq!(
            record.auth_results.dkim,
            Some(vec![DkimAuthResultType {
                domain: String::from("example.com"),
                selector: Some(String::from("ExamplesSelector")),
                result: DkimResultType::Fail,
                human_result: Some(String::from("Incoherent example"))
            }])
        );
        assert_eq!(
            record.auth_results.spf,
            vec![SpfAuthResultType {
                domain: String::from("example.com"),
                scope: Some(SpfDomainScope::Helo),
                result: SpfResultType::Pass,
            }]
        );
    }

    #[test]
    fn solamora_report() {
        let reader = File::open("testdata/dmarc-reports/solamora.xml").unwrap();
        let report: Report = serde_xml_rs::from_reader(reader).unwrap();

        // Check metadata
        assert_eq!(report.report_metadata.org_name, "solarmora.com");
        assert_eq!(
            report.report_metadata.email,
            "noreply-dmarc-support@solarmora.com"
        );
        assert_eq!(
            report.report_metadata.extra_contact_info.as_deref(),
            Some("http://solarmora.com/dmarc/support")
        );
        assert_eq!(report.report_metadata.report_id, "9391651994964116463");
        assert_eq!(report.report_metadata.date_range.begin, 1335571200);
        assert_eq!(report.report_metadata.date_range.end, 1335657599);

        // Check policy
        assert_eq!(report.policy_published.domain, "bix-business.com");
        assert_eq!(report.policy_published.adkim, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.aspf, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.p, DispositionType::None);
        assert_eq!(report.policy_published.sp, Some(DispositionType::None));
        assert_eq!(report.policy_published.pct, 100);

        // Check record
        assert_eq!(report.record.len(), 1);
        let record = report.record.first().unwrap();
        assert_eq!(record.row.source_ip.to_string(), "203.0.113.209");
        assert_eq!(record.row.count, 2);
        assert_eq!(
            record.row.policy_evaluated.disposition,
            DispositionType::None
        );
        assert_eq!(
            record.row.policy_evaluated.dkim,
            Some(DmarcResultType::Fail)
        );
        assert_eq!(record.row.policy_evaluated.spf, Some(DmarcResultType::Pass));
        assert_eq!(record.identifiers.header_from, "bix-business.com");
        assert_eq!(
            record.auth_results.dkim,
            Some(vec![DkimAuthResultType {
                domain: String::from("bix-business.com"),
                selector: None,
                result: DkimResultType::Fail,
                human_result: Some(String::new())
            }])
        );
        assert_eq!(
            record.auth_results.spf,
            vec![SpfAuthResultType {
                domain: String::from("bix-business.com"),
                scope: None,
                result: SpfResultType::Pass,
            }]
        );
    }

    #[test]
    fn yahoo_report() {
        let reader = File::open("testdata/dmarc-reports/yahoo.xml").unwrap();
        let report: Report = serde_xml_rs::from_reader(reader).unwrap();

        // Check metadata
        assert_eq!(report.report_metadata.org_name, "Yahoo");
        assert_eq!(report.report_metadata.email, "dmarchelp@yahooinc.com");
        assert_eq!(report.report_metadata.report_id, "1709600619.487850");
        assert_eq!(report.report_metadata.date_range.begin, 1709510400);
        assert_eq!(report.report_metadata.date_range.end, 1709596799);

        // Check policy
        assert_eq!(report.policy_published.domain, "random.org");
        assert_eq!(report.policy_published.adkim, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.aspf, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.p, DispositionType::Reject);
        assert_eq!(report.policy_published.pct, 100);

        // Check record
        assert_eq!(report.record.len(), 1);
        let record = report.record.first().unwrap();
        assert_eq!(record.row.source_ip.to_string(), "1.2.3.4");
        assert_eq!(record.row.count, 1);
        assert_eq!(
            record.row.policy_evaluated.disposition,
            DispositionType::None
        );
        assert_eq!(
            record.row.policy_evaluated.dkim,
            Some(DmarcResultType::Pass)
        );
        assert_eq!(record.row.policy_evaluated.spf, Some(DmarcResultType::Pass));
        assert_eq!(record.identifiers.header_from, "random.org");
        assert_eq!(
            record.auth_results.dkim,
            Some(vec![DkimAuthResultType {
                domain: String::from("random.org"),
                selector: Some(String::from("abc")),
                result: DkimResultType::Pass,
                human_result: None
            }])
        );
        assert_eq!(
            record.auth_results.spf,
            vec![SpfAuthResultType {
                domain: String::from("random.org"),
                scope: None,
                result: SpfResultType::Pass,
            }]
        );
    }

    #[test]
    fn google_report() {
        let reader = File::open("testdata/dmarc-reports/google.xml").unwrap();
        let report: Report = serde_xml_rs::from_reader(reader).unwrap();

        // Check metadata
        assert_eq!(report.report_metadata.org_name, "google.com");
        assert_eq!(
            report.report_metadata.email,
            "noreply-dmarc-support@google.com"
        );
        assert_eq!(
            report.report_metadata.extra_contact_info,
            Some(String::from("https://support.google.com/a/answer/2466580"))
        );
        assert_eq!(report.report_metadata.report_id, "3166094538684628578");
        assert_eq!(report.report_metadata.date_range.begin, 1709683200);
        assert_eq!(report.report_metadata.date_range.end, 1709769599);

        // Check policy
        assert_eq!(report.policy_published.domain, "foo-bar.io");
        assert_eq!(report.policy_published.adkim, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.aspf, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.p, DispositionType::Reject);
        assert_eq!(report.policy_published.sp, Some(DispositionType::Reject));
        assert_eq!(report.policy_published.pct, 100);

        // Check record
        assert_eq!(report.record.len(), 1);
        let record = report.record.first().unwrap();
        assert_eq!(record.row.source_ip.to_string(), "1.2.3.4");
        assert_eq!(record.row.count, 1);
        assert_eq!(
            record.row.policy_evaluated.disposition,
            DispositionType::None
        );
        assert_eq!(
            record.row.policy_evaluated.dkim,
            Some(DmarcResultType::Pass)
        );
        assert_eq!(record.row.policy_evaluated.spf, Some(DmarcResultType::Pass));
        assert_eq!(record.identifiers.header_from, "foo-bar.io");
        assert_eq!(
            record.auth_results.dkim,
            Some(vec![DkimAuthResultType {
                domain: String::from("foo-bar.io"),
                selector: Some(String::from("krs")),
                result: DkimResultType::Pass,
                human_result: None
            }])
        );
        assert_eq!(
            record.auth_results.spf,
            vec![SpfAuthResultType {
                domain: String::from("foo-bar.io"),
                scope: None,
                result: SpfResultType::Pass,
            }]
        );
    }

    #[test]
    fn outlook_report() {
        let reader = File::open("testdata/dmarc-reports/outlook.xml").unwrap();
        let report: Report = serde_xml_rs::from_reader(reader).unwrap();

        // Check metadata
        assert_eq!(report.report_metadata.org_name, "Outlook.com");
        assert_eq!(report.report_metadata.email, "dmarcreport@microsoft.com");
        assert_eq!(
            report.report_metadata.report_id,
            "a4f4ef0654474d3faa5dca167a34a86a"
        );
        assert_eq!(report.report_metadata.date_range.begin, 1709683200);
        assert_eq!(report.report_metadata.date_range.end, 1709769600);

        // Check policy
        assert_eq!(report.policy_published.domain, "random.net");
        assert_eq!(report.policy_published.adkim, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.aspf, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.p, DispositionType::Reject);
        assert_eq!(report.policy_published.sp, Some(DispositionType::Reject));
        assert_eq!(report.policy_published.pct, 100);
        assert_eq!(report.policy_published.fo, Some(String::from("0")));

        // Check record #1
        assert_eq!(report.record.len(), 2);
        let record = report.record.first().unwrap();
        assert_eq!(record.row.source_ip.to_string(), "1.2.3.4");
        assert_eq!(record.row.count, 1);
        assert_eq!(
            record.row.policy_evaluated.disposition,
            DispositionType::None
        );
        assert_eq!(
            record.row.policy_evaluated.dkim,
            Some(DmarcResultType::Pass)
        );
        assert_eq!(record.row.policy_evaluated.spf, Some(DmarcResultType::Pass));
        assert_eq!(
            record.identifiers.envelope_to,
            Some(String::from("live.de"))
        );
        assert_eq!(
            record.identifiers.envelope_from,
            Some(String::from("random.net"))
        );
        assert_eq!(record.identifiers.header_from, "random.net");
        assert_eq!(
            record.auth_results.dkim,
            Some(vec![DkimAuthResultType {
                domain: String::from("random.net"),
                selector: Some(String::from("def")),
                result: DkimResultType::Pass,
                human_result: None
            }])
        );
        assert_eq!(
            record.auth_results.spf,
            vec![SpfAuthResultType {
                domain: String::from("random.net"),
                scope: Some(SpfDomainScope::MailForm),
                result: SpfResultType::Pass,
            }]
        );

        // Check record #2
        let record = report.record.last().unwrap();
        assert_eq!(record.row.source_ip.to_string(), "1.2.3.4");
        assert_eq!(record.row.count, 2);
        assert_eq!(
            record.row.policy_evaluated.disposition,
            DispositionType::None
        );
        assert_eq!(
            record.row.policy_evaluated.dkim,
            Some(DmarcResultType::Pass)
        );
        assert_eq!(record.row.policy_evaluated.spf, Some(DmarcResultType::Pass));
        assert_eq!(
            record.identifiers.envelope_to,
            Some(String::from("outlook.de"))
        );
        assert_eq!(
            record.identifiers.envelope_from,
            Some(String::from("random.net"))
        );
        assert_eq!(record.identifiers.header_from, "random.net");
        assert_eq!(
            record.auth_results.dkim,
            Some(vec![DkimAuthResultType {
                domain: String::from("random.net"),
                selector: Some(String::from("def")),
                result: DkimResultType::Pass,
                human_result: None
            }])
        );
        assert_eq!(
            record.auth_results.spf,
            vec![SpfAuthResultType {
                domain: String::from("random.net"),
                scope: Some(SpfDomainScope::MailForm),
                result: SpfResultType::Pass,
            }]
        );
    }
}
