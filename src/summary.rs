use dmarc_aggregate_parser::aggregate_report::feedback;
use serde::Serialize;

#[derive(Serialize)]
pub struct Summary {
    /// Number of parsed DMARC reports from inbox
    pub reports: usize,
}

impl Summary {
    pub fn from_reports(reports: &[feedback]) -> Self {
        Self {
            reports: reports.len(),
        }
    }
}
