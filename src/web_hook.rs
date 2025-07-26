use crate::config::Configuration;
use crate::state::AppState;
use anyhow::{Context, Result, bail, ensure};
use axum::http::uri::Scheme;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::client::conn::http1;
use hyper::{Method, Request, Uri};
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_util::either::Either;
use tracing::{debug, error};

pub async fn mail_web_hook(
    config: &Configuration,
    mail_id: &str,
    state: &Arc<Mutex<AppState>>,
) -> Result<()> {
    let mail_details = get_mail_details(mail_id, state)
        .await
        .context("Failed to get mail details")?;

    let url = config
        .mail_web_hook_url
        .as_deref()
        .context("Failed to get web hook URL for new mails")?;

    // Inject mail details into URL in case it contains template parameters
    let url = inject_mail_details(&mail_details, url, true)
        .context("Failed to inject templates into URL")?;

    // Select HTTP method from config
    let method = Method::from_str(&config.mail_web_hook_method).context(format!(
        "Failed to parse string {} as HTTP method",
        config.mail_web_hook_method
    ))?;

    // Parse optional headers from config
    let mut header_map: HashMap<String, String> = HashMap::new();
    if let Some(headers) = &config.mail_web_hook_headers {
        header_map =
            serde_json::from_str(headers).context("Failed to parse optional header JSON")?;
    }

    // Parse and check URI
    let uri = url.parse::<Uri>().context("Failed to parse URL")?;
    let scheme = uri.scheme().context("URL has no scheme")?;
    ensure!(
        *scheme == Scheme::HTTP || *scheme == Scheme::HTTPS,
        "Only plain HTTP or HTTPS is supported"
    );

    // Get the host and the port
    let host = uri.host().context("URL has no host")?.to_string();
    let port = if let Some(port) = uri.port_u16() {
        port
    } else if *scheme == Scheme::HTTPS {
        443 // HTTPS
    } else {
        80 // HTTP
    };

    // Log details of hook call
    debug!("Calling web hook for new mail {mail_id} on URI {uri} with method {method}...");

    // Open a TCP or TLS connection to the remote host
    let stream = create_stream(&host, port, *scheme == Scheme::HTTPS)
        .await
        .context("Failed to create stream")?;

    // Create the Hyper client
    let io = TokioIo::new(stream);
    let (mut sender, conn) = http1::handshake(io)
        .await
        .context("Failed to create HTTP handshake")?;

    // Spawn a task to drive the HTTP state
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            error!("Connection failed: {err:?}");
        }
    });

    // Create HTTP request builder
    let mut req_builder = Request::builder()
        .uri(uri)
        .method(method)
        .header(hyper::header::HOST, host);

    // Append any custom headers
    for (key, val) in header_map {
        req_builder = req_builder.header(key, val);
    }

    // Prepare request body
    let body = Full::new(if let Some(body_str) = &config.mail_web_hook_body {
        let body_str = inject_mail_details(&mail_details, body_str, false)
            .context("Fauled to inject templates into mail body")?;
        Bytes::copy_from_slice(body_str.as_bytes())
    } else {
        Bytes::new()
    });

    // Finish building and send request
    let req = req_builder
        .body(body)
        .context("Failed to create HTTP request")?;
    let mut res = sender
        .send_request(req)
        .await
        .context("Failed to send HTTP request")?;

    let status_code = res.status().as_u16();
    debug!("Web hook for new mail {mail_id} responded with status code {status_code}");

    // Get response body piece by piece
    let mut body = Vec::new();
    while let Some(next) = res.frame().await {
        let frame = next.context("Failed to receive next HTTP response chunk")?;
        if let Some(chunk) = frame.data_ref() {
            body.extend_from_slice(chunk);
        }
        if body.len() > 1024 * 1024 {
            bail!("HTTP response too big");
        }
    }

    // Parse and log response body
    let body = String::from_utf8_lossy(&body);
    debug!("Web hook for new mail {mail_id} responded with body: {body}");

    Ok(())
}

fn inject_mail_details(
    details: &HashMap<&'static str, String>,
    template: &str,
    url_encode_value: bool,
) -> Result<String> {
    let mut template = template.to_string();
    for (key, value) in details {
        let placeholder = format!("[{key}]");
        let value = if url_encode_value {
            urlencoding::encode(value).to_string()
        } else {
            value.to_string()
        };
        template = template.replace(&placeholder, &value);
    }
    Ok(template)
}

async fn get_mail_details(
    mail_id: &str,
    state: &Arc<Mutex<AppState>>,
) -> Result<HashMap<&'static str, String>> {
    let locked_state = state.lock().await;
    let mail = locked_state
        .mails
        .get(mail_id)
        .context("Failed to find details for new mail")?;
    let dmarc_reports = locked_state
        .dmarc_reports
        .values()
        .filter(|r| r.mail_id == mail_id)
        .count();
    let tls_reports = locked_state
        .tls_reports
        .values()
        .filter(|r| r.mail_id == mail_id)
        .count();

    let mut result = HashMap::new();
    result.insert("id", mail_id.to_string());
    result.insert("uid", mail.uid.to_string());
    result.insert("sender", mail.sender.clone());
    result.insert("subject", mail.subject.clone());
    result.insert("folder", mail.folder.clone());
    result.insert("account", mail.account.clone());
    result.insert("dmarc_reports", dmarc_reports.to_string());
    result.insert("tls_reports", tls_reports.to_string());
    Ok(result)
}

async fn create_stream(
    host: &str,
    port: u16,
    tls: bool,
) -> Result<Either<TcpStream, TlsStream<TcpStream>>> {
    // Open a TCP connection to the remote host
    let address = format!("{host}:{port}");
    let tcp_stream = TcpStream::connect(&address)
        .await
        .context(format!("Failed to connect TCP stream at {address}"))?;

    // Early out in case of TCP without TLS
    if !tls {
        return Ok(Either::Left(tcp_stream));
    }

    // Create a TLS stream for HTTPS
    let cert_iter = webpki_roots::TLS_SERVER_ROOTS.iter().cloned();
    let root_cert_store = RootCertStore::from_iter(cert_iter);
    let client_config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(client_config));
    let dns_name =
        ServerName::try_from(host.to_string()).context("Failed to get DNS name from host")?;
    let tls_stream = connector
        .connect(dns_name, tcp_stream)
        .await
        .context("Failed to create TLS stream")?;
    Ok(Either::Right(tls_stream))
}
