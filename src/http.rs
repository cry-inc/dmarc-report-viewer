use crate::config::Configuration;
use crate::state::AppState;
use anyhow::{Context, Result};
use axum::{extract::State, routing::get, Router};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

pub async fn run_http_server(config: &Configuration, state: Arc<Mutex<AppState>>) -> Result<()> {
    let app = Router::new()
        .route("/", get(root))
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

async fn root(State(state): State<Arc<Mutex<AppState>>>) -> String {
    let mails = state.lock().expect("Failed to get app state lock").mails;
    format!("Hello World, we have {mails} mails")
}
