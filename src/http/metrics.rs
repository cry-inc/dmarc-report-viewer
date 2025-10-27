use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

pub async fn handler(State(_state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let lock = _state.lock().await;

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

    drop(lock);

    format!(
        "mails {mails}\n\
        xml_files {xml_files}\n\
        json_files {json_files}\n\
        dmarc_reports {dmarc_reports}\n\
        tls_reports {tls_reports}\n\
        last_update {last_update}\n\
        last_update_duration {last_update_duration}\n\
        start_time {start_time}\n\
        uptime {uptime}\n"
    )
}
