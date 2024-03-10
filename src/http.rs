use crate::config::Configuration;
use crate::state::AppState;
use crate::summary::Summary;
use anyhow::{Context, Result};
use axum::body::Body;
use axum::extract::Request;
use axum::http::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::{extract::State, routing::get, Router};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

pub async fn run_http_server(config: &Configuration, state: Arc<Mutex<AppState>>) -> Result<()> {
    let app = Router::new()
        .route("/", get(root))
        .route("/summary", get(summary))
        .route_layer(middleware::from_fn_with_state(
            config.clone(),
            basic_auth_middleware,
        ))
        .with_state(state.clone());

    let binding = format!("{}:{}", config.http_server_binding, config.http_server_port);
    info!("Starting HTTP server on binding {binding}...");

    let listener = TcpListener::bind(binding)
        .await
        .context("Failed to create TCP listener")?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Failed to serve HTTP with axum")
}

async fn shutdown_signal() {
    let ctrlc = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl + C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrlc => {},
        _ = terminate => {},
    }
}

async fn basic_auth_middleware(
    State(config): State<Configuration>,
    request: Request,
    next: Next,
) -> Response {
    // Password empty means basic auth is disabled
    if config.http_server_password.is_empty() {
        return next.run(request).await;
    }

    // Prepare error responses
    let unauthorized = Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(WWW_AUTHENTICATE, "Basic realm=\"Access\"")
        .body(Body::empty())
        .expect("Failed to create response");
    let bad_request = Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::empty())
        .expect("Failed to create response");

    let Some(header) = request.headers().get(AUTHORIZATION) else {
        return unauthorized;
    };
    let Ok(header) = header.to_str() else {
        return bad_request;
    };
    let Some(base64) = header.strip_prefix("Basic ") else {
        return bad_request;
    };
    let Ok(decoded) = STANDARD.decode(base64) else {
        return bad_request;
    };
    let Ok(string) = String::from_utf8(decoded) else {
        return bad_request;
    };
    let Some((user, password)) = string.split_once(':') else {
        return bad_request;
    };
    if user == config.http_server_user && password == config.http_server_password {
        next.run(request).await
    } else {
        unauthorized
    }
}

async fn root(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let mails = state.lock().expect("Failed to lock app state").mails;
    format!("Hello World, we have {mails} mails")
}

async fn summary(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let locked_state = state.lock().expect("Failed to lock app state");
    let summary = Summary::from_reports(&locked_state.reports);
    Json(summary)
}
