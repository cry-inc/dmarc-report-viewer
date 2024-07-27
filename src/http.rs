use crate::config::Configuration;
use crate::state::AppState;
use anyhow::{Context, Result};
use axum::body::Body;
use axum::extract::{Path, Request};
use axum::http::header::{self, AUTHORIZATION, WWW_AUTHENTICATE};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::IntoMakeService;
use axum::Json;
use axum::{extract::State, routing::get, Router};
use axum_server::Handle;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use futures::StreamExt;
use rustls_acme::caches::DirCache;
use rustls_acme::AcmeConfig;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::signal;
use tower_http::compression::CompressionLayer;
use tracing::{error, info, warn};

pub async fn run_http_server(config: &Configuration, state: Arc<Mutex<AppState>>) -> Result<()> {
    if config.http_server_password.is_empty() {
        warn!("Detected empty password: Basic Authentication will be disabled")
    }
    let make_service = Router::new()
        .route("/", get(index_html))
        .route("/chart.js", get(chart_js))
        .route("/lit.js", get(lit_js))
        .route("/components/app.js", get(comp_app_js))
        .route("/components/dashboard.js", get(comp_dashboard_js))
        .route("/components/reports.js", get(comp_reports_js))
        .route("/components/report.js", get(comp_report_js))
        .route("/summary", get(summary))
        .route("/reports", get(reports))
        .route("/reports/:id", get(report))
        .route_layer(CompressionLayer::new())
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

async fn index_html() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html")],
        #[cfg(debug_assertions)]
        std::fs::read("ui/index.html").expect("Failed to read index.html"),
        #[cfg(not(debug_assertions))]
        include_bytes!("../ui/index.html"),
    )
}

async fn comp_app_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript")],
        #[cfg(debug_assertions)]
        std::fs::read("ui/components/app.js").expect("Failed to read components/app.js"),
        #[cfg(not(debug_assertions))]
        include_bytes!("../ui/component/app.js"),
    )
}

async fn comp_dashboard_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript")],
        #[cfg(debug_assertions)]
        std::fs::read("ui/components/dashboard.js")
            .expect("Failed to read components/dashboard.js"),
        #[cfg(not(debug_assertions))]
        include_bytes!("../ui/component/dashboard.js"),
    )
}

async fn comp_reports_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript")],
        #[cfg(debug_assertions)]
        std::fs::read("ui/components/reports.js").expect("Failed to read components/reports.js"),
        #[cfg(not(debug_assertions))]
        include_bytes!("../ui/component/reports.js"),
    )
}

async fn comp_report_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript")],
        #[cfg(debug_assertions)]
        std::fs::read("ui/components/report.js").expect("Failed to read components/report.js"),
        #[cfg(not(debug_assertions))]
        include_bytes!("../ui/component/report.js"),
    )
}

async fn chart_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript")],
        include_bytes!("../ui/chart.umd.4.4.2.min.js"),
    )
}

async fn lit_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript")],
        include_bytes!("../ui/lit-core.3.1.4.min.js"),
    )
}

async fn summary(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    Json(
        state
            .lock()
            .expect("Failed to lock app state")
            .summary
            .clone(),
    )
}

#[derive(Serialize)]
struct ReportHeader {
    id: String,
    org: String,
    date_begin: u64,
    date_end: u64,
    rows: usize,
}

async fn reports(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let reports: Vec<ReportHeader> = state
        .lock()
        .expect("Failed to lock app state")
        .reports
        .iter()
        .map(|r| ReportHeader {
            id: r.report_metadata.report_id.clone(),
            org: r.report_metadata.org_name.clone(),
            date_begin: r.report_metadata.date_range.begin,
            date_end: r.report_metadata.date_range.end,
            rows: r.record.len(),
        })
        .collect();
    Json(reports)
}

async fn report(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let lock = state.lock().expect("Failed to lock app state");
    if let Some(report) = lock
        .reports
        .iter()
        .find(|r| *r.report_metadata.report_id == id)
    {
        let report_json = serde_json::to_string(report).expect("Failed to serialize JSON");
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            report_json,
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            format!("Cannot find report with ID {id}"),
        )
    }
}
