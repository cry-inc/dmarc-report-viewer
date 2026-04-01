use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

pub async fn handler(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let lock = state.lock().await;

    let mails = lock.mails.len();
    let xml_files = lock.xml_files;
    let json_files = lock.json_files;
    let dmarc_reports = lock.dmarc_reports.len();
    let tls_reports = lock.tls_reports.len();
    let start_time = lock.start_time;
    let uptime = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Failed to get Unix time stamp")
        .as_secs()
        - start_time;
    let last_update = lock.last_update;
    let last_update_duration = lock.last_update_duration;

    let dmarc_domains = get_dmarc_per_domain(&lock);
    let tls_domains = get_tls_per_domain(&lock);

    drop(lock);

    let dmarc_domains = format_labeled_metric(&dmarc_domains, "dmarc_reports", "domain");
    let tls_domains = format_labeled_metric(&tls_domains, "tls_reports", "domain");

    format!(
        "mails {mails}\n\
        xml_files {xml_files}\n\
        json_files {json_files}\n\
        dmarc_reports {dmarc_reports}\n\
        {dmarc_domains}\
        tls_reports {tls_reports}\n\
        {tls_domains}\
        last_update {last_update}\n\
        last_update_duration {last_update_duration}\n\
        start_time {start_time}\n\
        uptime {uptime}\n"
    )
}

fn get_dmarc_per_domain(state: &AppState) -> BTreeMap<String, usize> {
    let mut result = BTreeMap::new();
    for report in state.dmarc_reports.values() {
        let domain = report.report.policy_published.domain.clone();
        let value = result.entry(domain).or_insert(0);
        *value += 1;
    }
    result
}

fn get_tls_per_domain(state: &AppState) -> BTreeMap<String, usize> {
    let mut result = BTreeMap::new();
    for report in state.tls_reports.values() {
        for p in &report.report.policies {
            let domain = p.policy.policy_domain.clone();
            let value = result.entry(domain).or_insert(0);
            *value += 1;
        }
    }
    result
}

fn format_labeled_metric(
    data: &BTreeMap<String, usize>,
    metric_name: &str,
    label_name: &str,
) -> String {
    let mut result = String::new();
    for (label_value, value) in data {
        result.push_str(&format!(
            "{metric_name}{{{label_name}=\"{label_value}\"}} {value}\n"
        ));
    }
    result
}
