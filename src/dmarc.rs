// Original code before customization was copied from:
// https://github.com/bbustin/dmarc_aggregate_parser/
// Its based upon appendix C of the DMARC RFC:
// https://tools.ietf.org/html/rfc7489#appendix-C

use anyhow::{Context, Result};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::io::Cursor;
use std::net::IpAddr;

/// The time range in UTC covered by messages in this report.
/// Specified in seconds since epoch.
#[derive(Debug, Serialize, Deserialize)]
pub struct DateRangeType {
    pub begin: u64,
    pub end: u64,
}

/// Report generator metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReportMetadataType {
    pub org_name: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_contact_info: Option<String>,
    pub report_id: String,
    pub date_range: DateRangeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Vec<String>>,
}

/// Alignment mode for DKIM and SPF.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum AlignmentType {
    #[serde(rename = "r")]
    Relaxed,
    #[serde(rename = "s")]
    Strict,
}

/// The policy actions specified by `p` and `sp` in the DMARC record.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DispositionType {
    /// There is no preference on how a failed DMARC should be handled.
    None,
    /// The message should be quarantined.
    /// This usually means it will be placed in the spam folder of the user.
    Quarantine,
    /// The message should be rejected.
    Reject,
}

// Custom Deserialize to allow the empty string value that
// some reports contain. For some reason the serde alias marco
// does not work in that case.
impl<'de> Deserialize<'de> for DispositionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "quarantine" => Ok(Self::Quarantine),
            "reject" => Ok(Self::Reject),
            "none" => Ok(Self::None),
            "" => Ok(Self::None), // Some reports have an empty `sp` field
            _ => Err(de::Error::custom(format!(
                "'{s}' is not a known disposition type"
            ))),
        }
    }
}

/// The DMARC policy that applied to the messages in this report.
#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyPublishedType {
    /// The domain at which the DMARC record was found.
    pub domain: String,
    /// The DKIM alignment mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adkim: Option<AlignmentType>,
    /// The SPF alignment mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspf: Option<AlignmentType>,
    /// The policy to apply to messages from the domain.
    pub p: DispositionType,
    /// The policy to apply to messages from subdomains.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sp: Option<DispositionType>,
    /// The percent of messages to which policy applies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pct: Option<u8>,
    /// Failure reporting options in effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fo: Option<String>,
}

/// The DMARC-aligned authentication result.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DmarcResultType {
    Pass,
    Fail,
}

/// Reasons that may affect DMARC disposition or execution thereof.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyOverrideType {
    /// The message was relayed via a known forwarder, or local
    /// heuristics identified the message as likely having been forwarded.
    /// There is no expectation that authentication would pass.
    Forwarded,
    /// The message was exempted from application of policy by
    /// the `pct` setting in the DMARC policy record.
    SampledOut,
    /// Message authentication failure was anticipated by
    /// other evidence linking the message to a locally maintained list of
    /// known and trusted forwarders.
    TrustedForwarder,
    /// Local heuristics determined that the message arrived
    /// via a mailing list, and thus authentication of the original
    /// message was not expected to succeed.
    MailingList,
    /// The Mail Receiver's local policy exempted the message from
    /// being subjected to the Domain Owner's requested policy action.
    LocalPolicy,
    /// Some policy exception not covered by the other entries in
    /// this list occurred.  Additional detail can be found in the
    /// PolicyOverrideReason `comment` field.
    Other,
}

/// How do we allow report generators to include new classes of override
/// reasons if they want to be more specific than `other`?
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyOverrideReason {
    #[serde(rename = "type")]
    pub kind: PolicyOverrideType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Taking into account everything else in the record,
/// the results of applying DMARC.
#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyEvaluatedType {
    pub disposition: DispositionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dkim: Option<DmarcResultType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spf: Option<DmarcResultType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Vec<PolicyOverrideReason>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RowType {
    /// The connecting IP.
    pub source_ip: IpAddr,
    /// The number of matching messages.
    pub count: usize,
    /// The DMARC disposition applying to matching messages.
    pub policy_evaluated: PolicyEvaluatedType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentifierType {
    /// The envelope recipient domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub envelope_to: Option<String>,
    /// The RFC5321.MailFrom domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub envelope_from: Option<String>,
    /// The RFC5322.From domain.
    pub header_from: String,
}

/// DKIM verification result, according to RFC 7001 Section 2.6.1.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DkimAuthResultType {
    /// The `d` parameter in the signature.
    pub domain: String,
    /// The `s` parameter in the signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// The DKIM verification result.
    pub result: DkimResultType,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Any extra information.
    pub human_result: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SpfDomainScope {
    Helo,
    #[serde(rename = "mfrom")]
    MailForm,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpfResultType {
    None,
    Neutral,
    Pass,
    // Some reports use this value that is not really official, see issue #21
    #[serde(alias = "hardfail")]
    Fail,
    #[serde(rename = "softfail")]
    SoftFail,
    #[serde(rename = "temperror")]
    TemporaryError,
    #[serde(rename = "permerror")]
    PermanentError,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SpfAuthResultType {
    /// The checked domain.
    pub domain: String,
    /// The scope of the checked domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<SpfDomainScope>,
    /// The SPF verification result.
    pub result: SpfResultType,
}

/// This element contains DKIM and SPF results, uninterpreted with respect to DMARC.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResultType {
    /// There may be no DKIM signatures, or multiple DKIM signatures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dkim: Option<Vec<DkimAuthResultType>>,
    /// There will always be at least one SPF result.
    pub spf: Vec<SpfAuthResultType>,
}

/// This element contains all the authentication results that were
/// evaluated by the receiving system for the given set of messages.
#[derive(Debug, Serialize, Deserialize)]
pub struct RecordType {
    pub row: RowType,
    pub identifiers: IdentifierType,
    pub auth_results: AuthResultType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "feedback")]
pub struct Report {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub report_metadata: ReportMetadataType,
    pub policy_published: PolicyPublishedType,
    pub record: Vec<RecordType>,
}

impl Report {
    pub fn from_slice(xml_file: &[u8]) -> Result<Report> {
        let mut cursor = Cursor::new(xml_file);
        quick_xml::de::from_reader(&mut cursor).context("Failed to parse XML as DMARC report")
    }
}

#[derive(Serialize)]
pub struct DmarcParsingError {
    pub error: String,
    pub xml: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn serde_roundtrip() {
        let reader = BufReader::new(File::open("testdata/dmarc-reports/outlook.xml").unwrap());
        let report: Report = quick_xml::de::from_reader(reader).unwrap();

        let org_json = serde_json::to_string(&report).unwrap();
        let xml = quick_xml::se::to_string(&report).unwrap();

        let second: Report = quick_xml::de::from_str(&xml).unwrap();
        let sec_json = serde_json::to_string(&second).unwrap();
        assert_eq!(org_json, sec_json);

        let third: Report = serde_json::from_str(&org_json).unwrap();
        let third_json = serde_json::to_string(&third).unwrap();
        assert_eq!(org_json, third_json);
    }

    #[test]
    fn mailru_report() {
        let reader = BufReader::new(File::open("testdata/dmarc-reports/mailru.xml").unwrap());
        let report: Report = quick_xml::de::from_reader(reader).unwrap();

        // Check metadata
        assert_eq!(report.report_metadata.org_name, "Mail.Ru");
        assert_eq!(report.report_metadata.email, "dmarc_support@corp.mail.ru");
        assert_eq!(
            report.report_metadata.extra_contact_info.as_deref(),
            Some("http://help.mail.ru/mail-help")
        );
        assert_eq!(
            report.report_metadata.report_id,
            "28327321193681154911721360800"
        );
        assert_eq!(report.report_metadata.date_range.begin, 1721260800);
        assert_eq!(report.report_metadata.date_range.end, 1721347200);

        // Check policy
        assert_eq!(report.policy_published.domain, "foobar.de");
        assert_eq!(report.policy_published.adkim, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.aspf, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.p, DispositionType::Reject);
        assert_eq!(report.policy_published.sp, Some(DispositionType::Reject));
        assert_eq!(report.policy_published.pct, Some(100));

        // Check record
        assert_eq!(report.record.len(), 1);
        let record = report.record.first().unwrap();
        assert_eq!(record.row.source_ip.to_string(), "118.41.204.2");
        assert_eq!(record.row.count, 1);
        assert_eq!(
            record.row.policy_evaluated.disposition,
            DispositionType::Reject
        );
        assert_eq!(
            record.row.policy_evaluated.dkim,
            Some(DmarcResultType::Fail)
        );
        assert_eq!(record.row.policy_evaluated.spf, Some(DmarcResultType::Fail));
        assert_eq!(record.identifiers.header_from, "foobar.de");
        assert_eq!(record.auth_results.dkim, None);
        assert_eq!(
            record.auth_results.spf,
            vec![SpfAuthResultType {
                domain: String::from("foobar.de"),
                scope: Some(SpfDomainScope::MailForm),
                result: SpfResultType::SoftFail,
            }]
        );
    }

    #[test]
    fn aol_report() {
        let reader = BufReader::new(File::open("testdata/dmarc-reports/aol.xml").unwrap());
        let report: Report = quick_xml::de::from_reader(reader).unwrap();

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
        assert_eq!(report.policy_published.pct, Some(100));

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
        let reader = BufReader::new(File::open("testdata/dmarc-reports/acme.xml").unwrap());
        let report: Report = quick_xml::de::from_reader(reader).unwrap();

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
        assert_eq!(report.policy_published.pct, Some(100));
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
        let reader = BufReader::new(File::open("testdata/dmarc-reports/solamora.xml").unwrap());
        let report: Report = quick_xml::de::from_reader(reader).unwrap();

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
        assert_eq!(report.policy_published.pct, Some(100));

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
        let reader = BufReader::new(File::open("testdata/dmarc-reports/yahoo.xml").unwrap());
        let report: Report = quick_xml::de::from_reader(reader).unwrap();

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
        assert_eq!(report.policy_published.pct, Some(100));

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
        let reader = BufReader::new(File::open("testdata/dmarc-reports/google.xml").unwrap());
        let report: Report = quick_xml::de::from_reader(reader).unwrap();

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
        assert_eq!(report.policy_published.pct, Some(100));

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
        let reader = BufReader::new(File::open("testdata/dmarc-reports/outlook.xml").unwrap());
        let report: Report = quick_xml::de::from_reader(reader).unwrap();

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
        assert_eq!(report.policy_published.pct, Some(100));
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

    #[test]
    fn web_de_report() {
        let reader = BufReader::new(File::open("testdata/dmarc-reports/webde.xml").unwrap());
        let report: Report = quick_xml::de::from_reader(reader).unwrap();

        // Check metadata
        assert_eq!(report.report_metadata.org_name, "WEB.DE");
        assert_eq!(report.report_metadata.email, "noreply-dmarc@sicher.web.de");
        assert_eq!(
            report.report_metadata.report_id,
            "a3345c7cb5fd4f26aa62144bf449a54b"
        );
        assert_eq!(
            report.report_metadata.extra_contact_info.as_deref(),
            Some("https://postmaster.web.de/en/case?c=r2002")
        );
        assert_eq!(report.report_metadata.date_range.begin, 1722816000);
        assert_eq!(report.report_metadata.date_range.end, 1722902399);

        // Check policy
        assert_eq!(report.policy_published.domain, "foobar.com");
        assert_eq!(report.policy_published.adkim, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.aspf, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.p, DispositionType::Reject);
        assert_eq!(report.policy_published.sp, Some(DispositionType::None));
        assert_eq!(report.policy_published.pct, None);
        assert_eq!(report.policy_published.fo, None);

        // Check record
        assert_eq!(report.record.len(), 1);
        let record = report.record.first().unwrap();
        assert_eq!(record.row.source_ip.to_string(), "111.69.13.71");
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
        assert_eq!(record.identifiers.envelope_to, None);
        assert_eq!(
            record.identifiers.envelope_from.as_deref(),
            Some("foobar.com")
        );
        assert_eq!(record.identifiers.header_from, "foobar.com");
        assert_eq!(
            record.auth_results.dkim,
            Some(vec![DkimAuthResultType {
                domain: String::from("foobar.com"),
                selector: Some(String::from("sel123")),
                result: DkimResultType::Pass,
                human_result: None
            }])
        );
        assert_eq!(
            record.auth_results.spf,
            vec![SpfAuthResultType {
                domain: String::from("foobar.com"),
                scope: Some(SpfDomainScope::MailForm),
                result: SpfResultType::Pass,
            }]
        );
    }

    #[test]
    fn gmx_net_report() {
        let reader = BufReader::new(File::open("testdata/dmarc-reports/gmxnet.xml").unwrap());
        let report: Report = quick_xml::de::from_reader(reader).unwrap();

        // Check metadata
        assert_eq!(report.report_metadata.org_name, "GMX");
        assert_eq!(report.report_metadata.email, "noreply-dmarc@sicher.gmx.net");
        assert_eq!(
            report.report_metadata.report_id,
            "6d2be94cbabf4e838a3cf58fb4a42ab5"
        );
        assert_eq!(
            report.report_metadata.extra_contact_info.as_deref(),
            Some("https://postmaster.gmx.net/en/case?c=r2002")
        );
        assert_eq!(report.report_metadata.date_range.begin, 1733184000);
        assert_eq!(report.report_metadata.date_range.end, 1733270399);

        // Check policy
        assert_eq!(report.policy_published.domain, "myserver.com");
        assert_eq!(report.policy_published.adkim, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.aspf, Some(AlignmentType::Relaxed));
        assert_eq!(report.policy_published.p, DispositionType::Reject);
        assert_eq!(report.policy_published.sp, Some(DispositionType::Reject));

        // Check record
        assert_eq!(report.record.len(), 1);
        let record = report.record.first().unwrap();
        assert_eq!(record.row.source_ip.to_string(), "11.222.33.44");
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
            record.identifiers.envelope_from.as_deref(),
            Some("myserver.com")
        );
        assert_eq!(record.identifiers.header_from, "myserver.com");
        assert_eq!(
            record.auth_results.dkim,
            Some(vec![DkimAuthResultType {
                domain: String::from("myserver.com"),
                selector: Some(String::from("abc123")),
                result: DkimResultType::Pass,
                human_result: None
            }])
        );
        assert_eq!(
            record.auth_results.spf,
            vec![SpfAuthResultType {
                domain: String::from("myserver.com"),
                scope: Some(SpfDomainScope::MailForm),
                result: SpfResultType::Pass,
            }]
        );
    }

    #[test]
    fn hardfail_alias() {
        // Some reports use the value "hardfail" as SPF auth result, see issue #21.
        // According to https://datatracker.ietf.org/doc/html/rfc7489#appendix-C
        // this is not a valid value, but we allow it as alias for "fail".
        let reader = BufReader::new(File::open("testdata/dmarc-reports/hardfail.xml").unwrap());
        let report: Report = quick_xml::de::from_reader(reader).unwrap();
        let record = report.record.first().unwrap();
        let spf_auth_res = record.auth_results.spf.first().unwrap();
        assert_eq!(spf_auth_res.result, SpfResultType::Fail);
    }
}
