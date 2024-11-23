use crate::config::Configuration;
use crate::mail::Mail;
use crate::report::{DkimResultType, DmarcResultType, Report, SpfResultType};
use crate::state::AppState;
use anyhow::{Context, Result};
use axum::body::Body;
use axum::extract::{Path, Query, Request};
use axum::http::header::{self, AUTHORIZATION, WWW_AUTHENTICATE};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::IntoMakeService;
use axum::Json;
use axum::{extract::State, routing::get, Router};
use axum_server::Handle;
use base64::{engine::general_purpose::STANDARD, Engine};
use futures::StreamExt;
use rustls_acme::caches::DirCache;
use rustls_acme::AcmeConfig;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::signal;
use tracing::{error, info, warn};

pub async fn run_http_server(config: &Configuration, state: Arc<Mutex<AppState>>) -> Result<()> {
    if config.http_server_password.is_empty() {
        warn!("Detected empty password: Basic Authentication will be disabled")
    }
    let make_service = Router::new()
        .route("/summary", get(summary))
        .route("/reports", get(reports))
        .route("/reports/:id", get(report))
        .route("/xml-errors", get(xml_errors))
        .route("/mails", get(mails))
        .route("/mails/:id", get(mail))
        .route("/", get(static_file)) // index.html
        .route("/*filepath", get(static_file)) // all other files
        .route("/build", get(build))
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

async fn static_file(req: Request) -> impl IntoResponse {
    let path = req.uri().path();
    for sf in STATIC_FILES {
        if sf.http_path == path {
            let mut mime_type = "application/octet-stream";
            for mt in MIME_TYPES {
                if sf.file_path.ends_with(mt.ext) {
                    mime_type = mt.mime_type;
                    break;
                }
            }
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime_type)],
                #[cfg(debug_assertions)]
                std::fs::read(sf.file_path).expect("Failed to read file"),
                #[cfg(not(debug_assertions))]
                sf._data,
            );
        }
    }
    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_TYPE, "text/plain")],
        #[cfg(debug_assertions)]
        b"File not found".to_vec(),
        #[cfg(not(debug_assertions))]
        b"File not found",
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

async fn build() -> impl IntoResponse {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "hash": option_env!("GITHUB_SHA").unwrap_or("n/a"),
        "ref": option_env!("GITHUB_REF_NAME").unwrap_or("n/a"),
    }))
}

#[derive(Serialize)]
struct ReportHeader {
    id: String,
    org: String,
    domain: String,
    date_begin: u64,
    date_end: u64,
    records: usize,
    flagged: bool,
}

impl ReportHeader {
    pub fn from_report(report: &Report) -> Self {
        Self {
            id: report.report_metadata.report_id.clone(),
            org: report.report_metadata.org_name.clone(),
            domain: report.policy_published.domain.clone(),
            date_begin: report.report_metadata.date_range.begin,
            date_end: report.report_metadata.date_range.end,
            records: report.record.len(),
            flagged: Self::report_has_problem(report),
        }
    }

    fn report_has_problem(report: &Report) -> bool {
        report.record.iter().any(|record| {
            let policy_dkim = if let Some(dkim) = &record.row.policy_evaluated.dkim {
                *dkim != DmarcResultType::Pass
            } else {
                false
            };
            let policy_spf = if let Some(spf) = &record.row.policy_evaluated.spf {
                *spf != DmarcResultType::Pass
            } else {
                false
            };
            let dkim = if let Some(dkim) = &record.auth_results.dkim {
                dkim.iter().any(|x| x.result != DkimResultType::Pass)
            } else {
                false
            };
            let spf = record
                .auth_results
                .spf
                .iter()
                .any(|x| x.result != SpfResultType::Pass);
            dkim || spf || policy_dkim || policy_spf
        })
    }
}

#[derive(Deserialize)]
struct ReportFilters {
    uid: Option<u32>,
    flagged: Option<bool>,
    domain: Option<String>,
    org: Option<String>,
}

impl ReportFilters {
    fn decode(&self) -> Self {
        Self {
            uid: self.uid,
            flagged: self.flagged,
            domain: self
                .domain
                .as_ref()
                .and_then(|d| urlencoding::decode(d).ok())
                .map(|d| d.to_string()),
            org: self
                .org
                .as_ref()
                .and_then(|o| urlencoding::decode(o).ok())
                .map(|o| o.to_string()),
        }
    }
}

async fn reports(
    State(state): State<Arc<Mutex<AppState>>>,
    filters: Query<ReportFilters>,
) -> impl IntoResponse {
    // Remove URL encoding from strings in filters
    let filters = filters.decode();

    let reports: Vec<ReportHeader> = state
        .lock()
        .expect("Failed to lock app state")
        .reports
        .iter()
        .filter(|(uid, _)| {
            if let Some(queried_uid) = filters.uid {
                *uid == queried_uid
            } else {
                true
            }
        })
        .filter(|(_, r)| {
            if let Some(org) = &filters.org {
                r.report_metadata.org_name == *org
            } else {
                true
            }
        })
        .filter(|(_, r)| {
            if let Some(domain) = &filters.domain {
                r.policy_published.domain == *domain
            } else {
                true
            }
        })
        .map(|(uid, r)| (*uid, ReportHeader::from_report(r)))
        .filter(|(_, r)| {
            if let Some(flagged) = &filters.flagged {
                r.flagged == *flagged
            } else {
                true
            }
        })
        .map(|(_, r)| r)
        .collect();
    Json(reports)
}

async fn report(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let lock = state.lock().expect("Failed to lock app state");
    if let Some((_, report)) = lock
        .reports
        .iter()
        .find(|(_, r)| r.report_metadata.report_id == id)
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

async fn mail(
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
    let lock = state.lock().expect("Failed to lock app state");
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

async fn xml_errors(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let lock = state.lock().expect("Failed to lock app state");
    let errors_json = serde_json::to_string(&lock.xml_errors).expect("Failed to serialize JSON");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        errors_json,
    )
}

#[derive(Deserialize, Debug)]
struct MailFilters {
    sender: Option<String>,
    count: Option<usize>,
    oversized: Option<bool>,
}

impl MailFilters {
    fn decode(&self) -> Self {
        Self {
            oversized: self.oversized,
            count: self.count,
            sender: self
                .sender
                .as_ref()
                .and_then(|s| urlencoding::decode(s).ok())
                .map(|s| s.to_string()),
        }
    }
}

async fn mails(
    State(state): State<Arc<Mutex<AppState>>>,
    filters: Query<MailFilters>,
) -> impl IntoResponse {
    // Remove URL encoding from strings in filters
    let filters = filters.decode();

    let lock = state.lock().expect("Failed to lock app state");
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
                m.xml_file_count == *queried_count
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

const STATIC_FILES: &[StaticFile] = &[
    StaticFile {
        http_path: "/",
        file_path: "ui/index.html",
        _data: include_bytes!("../ui/index.html"),
    },
    StaticFile {
        http_path: "/chart.js",
        file_path: "ui/chart.umd.4.4.2.min.js",
        _data: include_bytes!("../ui/chart.umd.4.4.2.min.js"),
    },
    StaticFile {
        http_path: "/lit.js",
        file_path: "ui/lit-core.3.1.4.min.js",
        _data: include_bytes!("../ui/lit-core.3.1.4.min.js"),
    },
    StaticFile {
        http_path: "/components/app.js",
        file_path: "ui/components/app.js",
        _data: include_bytes!("../ui/components/app.js"),
    },
    StaticFile {
        http_path: "/components/dashboard.js",
        file_path: "ui/components/dashboard.js",
        _data: include_bytes!("../ui/components/dashboard.js"),
    },
    StaticFile {
        http_path: "/components/mailtable.js",
        file_path: "ui/components/mailtable.js",
        _data: include_bytes!("../ui/components/mailtable.js"),
    },
    StaticFile {
        http_path: "/components/problems.js",
        file_path: "ui/components/problems.js",
        _data: include_bytes!("../ui/components/problems.js"),
    },
    StaticFile {
        http_path: "/components/report.js",
        file_path: "ui/components/report.js",
        _data: include_bytes!("../ui/components/report.js"),
    },
    StaticFile {
        http_path: "/components/reports.js",
        file_path: "ui/components/reports.js",
        _data: include_bytes!("../ui/components/reports.js"),
    },
    StaticFile {
        http_path: "/components/mails.js",
        file_path: "ui/components/mails.js",
        _data: include_bytes!("../ui/components/mails.js"),
    },
    StaticFile {
        http_path: "/components/mail.js",
        file_path: "ui/components/mail.js",
        _data: include_bytes!("../ui/components/mail.js"),
    },
    StaticFile {
        http_path: "/components/about.js",
        file_path: "ui/components/about.js",
        _data: include_bytes!("../ui/components/about.js"),
    },
    StaticFile {
        http_path: "/components/reporttable.js",
        file_path: "ui/components/reporttable.js",
        _data: include_bytes!("../ui/components/reporttable.js"),
    },
];

const MIME_TYPES: &[MimeType] = &[
    MimeType {
        ext: ".html",
        mime_type: "text/html",
    },
    MimeType {
        ext: ".js",
        mime_type: "text/javascript",
    },
];

struct MimeType {
    ext: &'static str,
    mime_type: &'static str,
}

struct StaticFile {
    http_path: &'static str,
    file_path: &'static str,
    _data: &'static [u8],
}
