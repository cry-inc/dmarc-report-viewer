use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The time range covered by messages in this report.
/// Formatted according to "Internet Date/Time Format", Section 5.6 of RFC3339.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DateRange {
    pub start_datetime: DateTime<Utc>,
    pub end_datetime: DateTime<Utc>,
}

/// Type of the policy evaluated by the reporting organization.
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyType {
    /// The MTA-STS Policy was applied.
    Sts,
    /// The DANE TLSA record was applied.
    Tlsa,
    /// Neither a DANE nor an MTA-STS Policy could be found.
    NoPolicyFound,
}

/// The policy evaluated by the reporting organization.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Policy {
    /// The type of policy that was applied by the sending domain.
    pub policy_type: PolicyType,
    /// An encoding of the applied policy as a JSON array of strings, whether
    /// it is a TLSA record or an MTA-STS Policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_string: Option<Vec<String>>,
    /// The Policy Domain against which the MTA-STS or DANE policy is defined.
    pub policy_domain: String,
    /// In the case where "policy-type" is "sts": the pattern of MX hostnames
    /// from the applied policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mx_host: Option<Vec<String>>,
}

/// A summary of the policy evaluation result.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Summary {
    pub total_successful_session_count: u64,
    pub total_failure_session_count: u64,
}

/// The result type, describing what went wrong during the connection attempt.
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ResultType {
    // Negotiation failures
    /// The recipient MX did not support STARTTLS.
    StarttlsNotSupported,
    /// The certificate presented did not adhere to the constraints specified
    /// in the MTA-STS or DANE policy, e.g., if the MX hostname does not match
    /// any identities listed in the subject alternative name (SAN).
    CertificateHostMismatch,
    /// The certificate has expired.
    CertificateExpired,
    /// A label that covers multiple certificate-related failures that include,
    /// but are not limited to errors such as untrusted/unknown certification
    /// authorities (CAs), certificate name constraints, certificate chain
    /// errors, etc.
    CertificateNotTrusted,
    /// A general failure for a reason not matching a category above.
    ValidationFailure,

    // Policy failures – DANE-specific
    /// A validation error in the TLSA recurd associated with a DANE policy.
    TlsaInvalid,
    /// No valid records were returned from the recursive resolver.
    DnssecInvalid,
    /// The sending system is configured to require DANE TLSA records for all
    /// the MX hosts of the destination domain, but no DNSSEC-validated TLSA
    /// records were present for the MX host that is the subject of the report.
    DaneRequired,

    // Policy failures – MTA-STS-specific
    /// A failure to retrieve an MTA-STS policy, for example, because the policy
    /// host is unreachable.
    StsPolicyFetchError,
    /// A validation error for the overall MTA-STS policy.
    StsPolicyInvalid,
    /// The MTA-STS policy could not be authenticated using PKIX validation.
    StsWebpkiInvalid,
}

/// Aggregated failure details for a single result type.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FailureDetails {
    /// The type of the failure.
    pub result_type: ResultType,
    /// The IP address of the Sending MTA that attempted the STARTTLS
    /// connection.
    pub sending_mta_ip: String,
    /// The hostname of the receiving MTA MX record with which the Sending MTA
    /// attempted to negotiate a STARTTLS connection.
    pub receiving_mx_hostname: String,
    /// The HELLO (HELO) or Extended HELLO (EHLO) string from the banner
    /// announced during the reported session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiving_mx_helo: Option<String>,
    /// The destination IP address that was used when creating the outbound
    /// session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiving_ip: Option<String>,
    /// The number of (attempted) sessions that match the relevant
    /// "result-type" for this section.
    pub failed_session_count: u64,
    /// A URI that points to additional information around the relevant
    /// "result-type".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<String>,
    /// A text field to include a TLS-related error code or error message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason_code: Option<String>,
}

/// An evaluation result for a single policy.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PolicyResult {
    /// The policy that was evaluated.
    pub policy: Policy,
    /// The summary of the policy evaluation result.
    pub summary: Summary,
    /// A list of aggregated failure details that occurred during the
    /// evaluation of the policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_details: Option<Vec<FailureDetails>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Report {
    /// The name of the organization responsible for the report.
    pub organization_name: String,
    /// The date-time indicating the start and end times for the report range.
    pub date_range: DateRange,
    /// The contact information for the party responsible for the report.
    pub contact_info: String,
    /// A unique identifier for the report.
    pub report_id: String,
    /// The policies evaluated by the reporting organization.
    pub policies: Vec<PolicyResult>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn serde_roundtrip() {
        let reader =
            BufReader::new(File::open("testdata/tls-rpt-reports/rfc-example.json").unwrap());
        let report_1: Report = serde_json::from_reader(reader).unwrap();
        let string_1 = serde_json::to_string(&report_1).unwrap();

        let report_2: Report = serde_json::from_str(&string_1).unwrap();
        let string_2 = serde_json::to_string(&report_2).unwrap();
        assert_eq!(string_1, string_2);

        let report_3: Report = serde_json::from_str(&string_2).unwrap();
        let string_3 = serde_json::to_string(&report_3).unwrap();
        assert_eq!(string_2, string_3);
    }

    #[test]
    fn rfc_example_report() {
        let reader =
            BufReader::new(File::open("testdata/tls-rpt-reports/rfc-example.json").unwrap());
        let report: Report = serde_json::from_reader(reader).unwrap();

        // Check metadata
        assert_eq!(report.organization_name, "Company-X");
        assert_eq!(
            report.date_range.start_datetime.to_rfc3339(),
            "2016-04-01T00:00:00+00:00"
        );
        assert_eq!(
            report.date_range.end_datetime.to_rfc3339(),
            "2016-04-01T23:59:59+00:00"
        );
        assert_eq!(report.contact_info, "sts-reporting@company-x.example");
        assert_eq!(report.report_id, "5065427c-23d3-47ca-b6e0-946ea0e8c4be");

        // Check policy results
        assert_eq!(report.policies.len(), 1);
        let policy_result = &report.policies[0];

        // Check policy result – policy
        assert_eq!(policy_result.policy.policy_type, PolicyType::Sts);
        assert_eq!(
            policy_result.policy.policy_string,
            Some(vec![
                "version: STSv1".to_string(),
                "mode: testing".to_string(),
                "mx: *.mail.company-y.example".to_string(),
                "max_age: 86400".to_string(),
            ])
        );
        assert_eq!(policy_result.policy.policy_domain, "company-y.example");
        assert_eq!(
            policy_result.policy.mx_host,
            Some(vec!["*.mail.company-y.example".to_string()])
        );

        // Check policy result – summary
        assert_eq!(policy_result.summary.total_successful_session_count, 5326);
        assert_eq!(policy_result.summary.total_failure_session_count, 303);

        // Check policy result – failure details
        assert_eq!(policy_result.failure_details.as_ref().unwrap().len(), 3);
        let cert_expired_details = &policy_result.failure_details.as_ref().unwrap()[0];
        let no_starttls_details = &policy_result.failure_details.as_ref().unwrap()[1];
        let validation_failure_details = &policy_result.failure_details.as_ref().unwrap()[2];

        assert_eq!(
            cert_expired_details.result_type,
            ResultType::CertificateExpired
        );
        assert_eq!(cert_expired_details.sending_mta_ip, "2001:db8:abcd:0012::1");
        assert_eq!(
            cert_expired_details.receiving_mx_hostname,
            "mx1.mail.company-y.example"
        );
        assert!(cert_expired_details.receiving_mx_helo.is_none());
        assert!(cert_expired_details.receiving_ip.is_none());
        assert_eq!(cert_expired_details.failed_session_count, 100);
        assert!(cert_expired_details.additional_information.is_none());
        assert!(cert_expired_details.failure_reason_code.is_none());

        assert_eq!(
            no_starttls_details.result_type,
            ResultType::StarttlsNotSupported
        );
        assert_eq!(no_starttls_details.sending_mta_ip, "2001:db8:abcd:0013::1");
        assert_eq!(
            no_starttls_details.receiving_mx_hostname,
            "mx2.mail.company-y.example"
        );
        assert!(no_starttls_details.receiving_mx_helo.is_none());
        assert_eq!(
            no_starttls_details.receiving_ip,
            Some("203.0.113.56".to_string())
        );
        assert_eq!(no_starttls_details.failed_session_count, 200);
        assert_eq!(no_starttls_details.additional_information, Some("https://reports.company-x.example/report_info?id=5065427c-23d3#StarttlsNotSupported".to_string()));
        assert!(no_starttls_details.failure_reason_code.is_none());

        assert_eq!(
            validation_failure_details.result_type,
            ResultType::ValidationFailure
        );
        assert_eq!(validation_failure_details.sending_mta_ip, "198.51.100.62");
        assert_eq!(
            validation_failure_details.receiving_mx_hostname,
            "mx-backup.mail.company-y.example"
        );
        assert!(validation_failure_details.receiving_mx_helo.is_none());
        assert_eq!(
            validation_failure_details.receiving_ip,
            Some("203.0.113.58".to_string())
        );
        assert_eq!(validation_failure_details.failed_session_count, 3);
        assert!(validation_failure_details.additional_information.is_none());
        assert_eq!(
            validation_failure_details.failure_reason_code,
            Some("X509_V_ERR_PROXY_PATH_LENGTH_EXCEEDED".to_string())
        );
    }

    #[test]
    fn microsoft_report() {
        let reader = BufReader::new(File::open("testdata/tls-rpt-reports/microsoft.json").unwrap());
        let report: Report = serde_json::from_reader(reader).unwrap();

        // Check metadata
        assert_eq!(report.organization_name, "Microsoft Corporation");
        assert_eq!(
            report.date_range.start_datetime.to_rfc3339(),
            "2025-05-23T00:00:00+00:00"
        );
        assert_eq!(
            report.date_range.end_datetime.to_rfc3339(),
            "2025-05-23T23:59:59+00:00"
        );
        assert_eq!(report.contact_info, "tlsrpt-noreply@microsoft.com");
        assert_eq!(report.report_id, "133925885310113267+random.net");

        // Check policy results
        assert_eq!(report.policies.len(), 2);
        let sts_policy_result = &report.policies[0];
        let tlsa_policy_result = &report.policies[1];

        // Check STS policy result – policy
        assert_eq!(sts_policy_result.policy.policy_type, PolicyType::Sts);
        assert_eq!(
            sts_policy_result.policy.policy_string,
            Some(vec![
                "version: STSv1".to_string(),
                "mode: enforce".to_string(),
                "mx: *.random.net".to_string(),
                "max_age: 2592000".to_string(),
            ])
        );
        assert_eq!(sts_policy_result.policy.policy_domain, "random.net");
        assert!(sts_policy_result.policy.mx_host.is_none());

        // Check STS policy result – summary
        assert_eq!(sts_policy_result.summary.total_successful_session_count, 2);
        assert_eq!(sts_policy_result.summary.total_failure_session_count, 0);

        // Check STS policy result – failure details
        assert!(sts_policy_result.failure_details.is_none());

        // Check TLSA policy result – policy
        assert_eq!(tlsa_policy_result.policy.policy_type, PolicyType::Tlsa);
        assert_eq!(tlsa_policy_result.policy.policy_string, Some(vec![
            "[\"3 1 1 6007EEE553E85D8DF007A845D19EC343283D4E416E9A33F9EF3040C8B7C285BC\",\"3 1 1 837C773D54C2E2BD71871A3FC352BE8214D5646CBAE5E3091401A7274717998B\"]".to_string(),
        ]));
        assert_eq!(tlsa_policy_result.policy.policy_domain, "random.net");
        assert!(tlsa_policy_result.policy.mx_host.is_none());

        // Check TLSA policy result – summary
        assert_eq!(tlsa_policy_result.summary.total_successful_session_count, 2);
        assert_eq!(tlsa_policy_result.summary.total_failure_session_count, 0);

        // Check TLSA policy result – failure details
        assert!(tlsa_policy_result.failure_details.is_none());
    }

    #[test]
    fn google_report() {
        let reader = BufReader::new(File::open("testdata/tls-rpt-reports/google.json").unwrap());
        let report: Report = serde_json::from_reader(reader).unwrap();

        // Check metadata
        assert_eq!(report.organization_name, "Google Inc.");
        assert_eq!(
            report.date_range.start_datetime.to_rfc3339(),
            "2025-05-22T00:00:00+00:00"
        );
        assert_eq!(
            report.date_range.end_datetime.to_rfc3339(),
            "2025-05-22T23:59:59+00:00"
        );
        assert_eq!(report.contact_info, "smtp-tls-reporting@google.com");
        assert_eq!(report.report_id, "2025-05-22T00:00:00Z_foo-bar.io");

        // Check policy results
        assert_eq!(report.policies.len(), 1);
        let policy_result = &report.policies[0];

        // Check policy result – policy
        assert_eq!(policy_result.policy.policy_type, PolicyType::Sts);
        assert_eq!(
            policy_result.policy.policy_string,
            Some(vec![
                "version: STSv1".to_string(),
                "mode: enforce".to_string(),
                "mx: *.foo-bar.io".to_string(),
                "max_age: 2592000".to_string(),
            ])
        );
        assert_eq!(policy_result.policy.policy_domain, "foo-bar.io");
        assert_eq!(
            policy_result.policy.mx_host,
            Some(vec!["*.foo-bar.io".to_string()])
        );

        // Check policy result – summary
        assert_eq!(policy_result.summary.total_successful_session_count, 1);
        assert_eq!(policy_result.summary.total_failure_session_count, 0);

        // Check policy result – failure details
        assert!(policy_result.failure_details.is_none());
    }

    #[test]
    fn no_policy_report() {
        let reader = BufReader::new(File::open("testdata/tls-rpt-reports/no-policy.json").unwrap());
        let report: Report = serde_json::from_reader(reader).unwrap();

        // Check metadata
        assert_eq!(report.organization_name, "Google Inc.");
        assert_eq!(
            report.date_range.start_datetime.to_rfc3339(),
            "2025-03-27T00:00:00+00:00"
        );
        assert_eq!(
            report.date_range.end_datetime.to_rfc3339(),
            "2025-03-27T23:59:59+00:00"
        );
        assert_eq!(report.contact_info, "smtp-tls-reporting@google.com");
        assert_eq!(report.report_id, "2025-03-27T00:00:00Z_foo-bar.io");

        // Check policy results
        assert_eq!(report.policies.len(), 1);
        let policy_result = &report.policies[0];

        // Check policy result – policy
        assert_eq!(policy_result.policy.policy_type, PolicyType::NoPolicyFound);
        assert!(policy_result.policy.policy_string.is_none());
        assert_eq!(policy_result.policy.policy_domain, "foo-bar.io");
        assert!(policy_result.policy.mx_host.is_none());

        // Check policy result – summary
        assert_eq!(policy_result.summary.total_successful_session_count, 1);
        assert_eq!(policy_result.summary.total_failure_session_count, 0);
    }
}
