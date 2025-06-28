use crate::mail::Mail;
use crate::state::AppState;
use axum::extract::State;
use axum::extract::{Path, Query};
use axum::http::header;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn single_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let lock = state.lock().await;
    if let Some((_, mail)) = lock.mails.iter().find(|(i, _)| **i == id) {
        let mail_json = serde_json::to_string(mail).expect("Failed to serialize JSON");
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            mail_json,
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            String::from("Cannot find mail"),
        )
    }
}

pub async fn errors_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let lock = state.lock().await;
    if !lock.mails.contains_key(&id) {
        return (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            String::from("Cannot find mail"),
        );
    }
    let empty = Vec::new();
    let errors = lock.parsing_errors.get(&id).unwrap_or(&empty);
    let errors_json = serde_json::to_string(&errors).expect("Failed to serialize JSON");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        errors_json,
    )
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Attachment {
    Dmarc,
    Tls,
    None,
}

#[derive(Deserialize, Debug)]
pub struct MailFilters {
    sender: Option<String>,
    attachment: Option<Attachment>,
    oversized: Option<bool>,
    errors: Option<bool>,
}

impl MailFilters {
    fn url_decode(&mut self) {
        self.sender = self
            .sender
            .as_ref()
            .and_then(|s| urlencoding::decode(s).ok())
            .map(|s| s.to_string());
    }
}

pub async fn list_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    mut filters: Query<MailFilters>,
) -> impl IntoResponse {
    // Remove URL encoding from strings in filters
    filters.url_decode();

    let lock = state.lock().await;
    let mails: Vec<&Mail> = lock
        .mails
        .values()
        .filter(|m| {
            if let Some(queried_sender) = &filters.sender {
                m.sender == *queried_sender
            } else {
                true
            }
        })
        .filter(|m| {
            if let Some(queried_oversized) = &filters.oversized {
                m.oversized == *queried_oversized
            } else {
                true
            }
        })
        .filter(|m| {
            if let Some(queried_type) = &filters.attachment {
                match queried_type {
                    Attachment::Dmarc => m.xml_files > 0,
                    Attachment::Tls => m.json_files > 0,
                    Attachment::None => m.xml_files == 0 && m.json_files == 0,
                }
            } else {
                true
            }
        })
        .filter(|m| {
            if let Some(queried_errors) = &filters.errors {
                (m.xml_parsing_errors > 0 || m.json_parsing_errors > 0) == *queried_errors
            } else {
                true
            }
        })
        .collect();
    let mails_json = serde_json::to_string(&mails).expect("Failed to serialize JSON");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        mails_json,
    )
}
