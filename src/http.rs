mod dmarc_reports;
mod ips;
mod mails;
mod sources;
mod static_files;
mod summary;
mod tls_reports;

use crate::config::Configuration;
use crate::state::AppState;
use anyhow::{Context, Result};
use axum::Json;
use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::http::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{IntoMakeService, get, post};
use axum::{Router, extract::State};
use axum_server::Handle;
use base64::{Engine, engine::general_purpose::STANDARD};
use futures::StreamExt;
use rustls_acme::AcmeConfig;
use rustls_acme::caches::DirCache;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

pub async fn run_http_server(config: &Configuration, state: Arc<Mutex<AppState>>) -> Result<()> {
    if config.http_server_password.is_empty() {
        warn!("Detected empty password: Basic Authentication will be disabled")
    }
    let make_service = Router::new()
        .route("/summary", get(summary::handler))
        .route("/mails", get(mails::list_handler))
        .route("/mails/{id}", get(mails::single_handler))
        .route("/mails/{id}/errors", get(mails::errors_handler))
        .route("/dmarc-reports", get(dmarc_reports::list_handler))
        .route("/dmarc-reports/{id}", get(dmarc_reports::single_handler))
        .route("/dmarc-reports/{id}/json", get(dmarc_reports::json_handler))
        .route("/dmarc-reports/{id}/xml", get(dmarc_reports::xml_handler))
        .route("/tls-reports", get(tls_reports::list_handler))
        .route("/tls-reports/{id}", get(tls_reports::single_handler))
        .route("/tls-reports/{id}/json", get(tls_reports::json_handler))
        .route("/sources", get(sources::handler))
        .route("/ips/{ip}/dns", get(ips::dns_single_handler))
        .route("/ips/dns/batch", post(ips::dns_batch_handler))
        .route("/ips/{ip}/location", get(ips::to_location_handler))
        .route("/ips/{ip}/whois", get(ips::to_whois_handler))
        .route("/build", get(build))
        .route("/", get(static_files::handler)) // index.html
        .route("/{*filepath}", get(static_files::handler)) // all other files
        .route_layer(middleware::from_fn_with_state(
            config.clone(),
            basic_auth_middleware,
        ))
        .with_state(state.clone())
        .into_make_service();

    let binding = format!("{}:{}", config.http_server_binding, config.http_server_port);
    let addr: SocketAddr = binding.parse().context("Failed to parse binding address")?;
    info!("Binding HTTP server to {addr}...");

    if config.https_auto_cert {
        start_https_server(config, addr, make_service)
            .await
            .context("Failed to start HTTPS server")
    } else {
        start_http_server(addr, make_service)
            .await
            .context("Failed to start HTTP server")
    }
}

async fn start_http_server(
    addr: SocketAddr,
    make_service: IntoMakeService<Router>,
) -> anyhow::Result<()> {
    let handle = Handle::new();
    let handle_clone = handle.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        handle_clone.shutdown();
    });

    axum_server::bind(addr)
        .handle(handle)
        .serve(make_service)
        .await
        .context("Failed to create axum HTTP server")
}

async fn start_https_server(
    config: &Configuration,
    addr: SocketAddr,
    make_service: IntoMakeService<Router>,
) -> anyhow::Result<()> {
    let handle = Handle::new();
    let handle_clone = handle.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        handle_clone.shutdown();
    });

    let acme_domain = config
        .https_auto_cert_domain
        .as_deref()
        .context("HTTPS automatic certificate domain is missing in configuration")?;

    let acme_contact = format!(
        "mailto:{}",
        config
            .https_auto_cert_mail
            .as_deref()
            .context("HTTPS automatic certificate mail is missing in configuration")?
    );

    let acme_cache = DirCache::new(
        config
            .https_auto_cert_cache
            .as_deref()
            .context("HTTPS automatic certificate cache directory is missing in configuration")?
            .to_owned(),
    );

    let mut acme_state = AcmeConfig::new([acme_domain])
        .contact([acme_contact])
        .cache_option(Some(acme_cache))
        .directory_lets_encrypt(true)
        .state();
    let rustls_config = acme_state.default_rustls_config();
    let acceptor = acme_state.axum_acceptor(rustls_config);

    tokio::spawn(async move {
        loop {
            match acme_state
                .next()
                .await
                .expect("Failed to get next ACME event")
            {
                Ok(ok) => info!("ACME Event: {:?}", ok),
                Err(err) => error!("ACME Error: {:?}", err),
            }
        }
    });

    axum_server::bind(addr)
        .handle(handle)
        .acceptor(acceptor)
        .serve(make_service)
        .await
        .context("Failed to create axum HTTPS server")
}

/// Promise will be fulfilled when a shutdown signal is received
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

/// Middleware to add basic auth password protection
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

async fn build() -> impl IntoResponse {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "hash": option_env!("GITHUB_SHA").unwrap_or("n/a"),
        "ref": option_env!("GITHUB_REF_NAME").unwrap_or("n/a"),
    }))
}
