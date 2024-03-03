use crate::config::Configuration;
use anyhow::{Context, Result};
use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tracing::info;

pub async fn run_http_server(config: &Configuration) -> Result<()> {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let binding = format!("{}:{}", config.http_server_binding, config.http_server_port);
    info!("Starting HTTP server on binding {binding}...");

    let listener = TcpListener::bind(binding)
        .await
        .context("Failed to create TCP listener")?;
    axum::serve(listener, app)
        .await
        .context("Failed to serve HTTP with axum")
}
