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
    let Ok(parsed_uid) = id.parse::<u32>() else {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "text/plain")],
            format!("Invalid ID {id}"),
        );
    };
    let lock = state.lock().await;
    if let Some((_, mail)) = lock.mails.iter().find(|(uid, _)| **uid == parsed_uid) {
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
            format!("Cannot find mail with ID {id}"),
        )
    }
}

pub async fn errors_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Ok(parsed_uid) = id.parse::<u32>() else {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "text/plain")],
            format!("Invalid ID {id}"),
        );
    };
    let lock = state.lock().await;
    if !lock.mails.contains_key(&parsed_uid) {
        return (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            format!("Cannot find mail with ID {id}"),
        );
    }
    if let Some(errors) = lock.dmarc_parsing_errors.get(&parsed_uid) {
        let errors_json = serde_json::to_string(errors).expect("Failed to serialize JSON");
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            errors_json,
        )
    } else {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            String::from("[]"),
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct MailFilters {
    sender: Option<String>,
    count: Option<usize>,
    oversized: Option<bool>,
    errors: Option<bool>,
}

impl MailFilters {
    fn url_decode(&self) -> Self {
        Self {
            oversized: self.oversized,
            count: self.count,
            errors: self.errors,
            sender: self
                .sender
                .as_ref()
                .and_then(|s| urlencoding::decode(s).ok())
                .map(|s| s.to_string()),
        }
    }
}

pub async fn list_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    filters: Query<MailFilters>,
) -> impl IntoResponse {
    // Remove URL encoding from strings in filters
    let filters = filters.url_decode();

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
            if let Some(queried_count) = &filters.count {
                m.xml_files == *queried_count
            } else {
                true
            }
        })
        .filter(|m| {
            if let Some(queried_errors) = &filters.errors {
                (m.parsing_errors > 0) == *queried_errors
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
