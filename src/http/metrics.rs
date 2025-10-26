use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handler(State(_state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let lock = _state.lock().await;

    let mails = lock.mails.len();
    let xml_files = lock.xml_files;
    let json_files = lock.json_files;
    let dmarc_reports = lock.dmarc_reports.len();
    let tls_reports = lock.tls_reports.len();

    format!(
        "mails {mails}\n\
        xml_files {xml_files}\n\
        json_files {json_files}\n\
        dmarc_reports {dmarc_reports}\n\
        tls_reports {tls_reports}\n"
    )
}
