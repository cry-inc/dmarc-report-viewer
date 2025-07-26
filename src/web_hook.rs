use crate::config::Configuration;
use anyhow::{Context, Result, bail, ensure};
use axum::http::uri::Scheme;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::client::conn::http1;
use hyper::{Method, Request, Uri};
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::str::FromStr;
use tokio::net::TcpStream;
use tracing::{debug, error};

pub async fn mail_web_hook(config: &Configuration, mail_id: &str) -> Result<()> {
    let url = config
        .mail_web_hook_url
        .as_deref()
        .context("Failed to get web hook URL for new mails")?;

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

    // Create and parse URI
    let uri = url.parse::<Uri>().context("Failed to parse URL")?;
    ensure!(
        uri.scheme().context("URL has no scheme")? == &Scheme::HTTP,
        "Only plain HTTP is supported"
    );

    // Get the host and the port
    let host = uri.host().context("URL has no host")?.to_string();
    let port = uri.port_u16().unwrap_or(80);

    // Log details of hook call
    debug!("Calling web hook for new mail {mail_id} on URI {uri} with method {method}...");

    // Open a TCP connection to the remote host
    let address = format!("{host}:{port}");
    let stream = TcpStream::connect(&address)
        .await
        .context(format!("Failed to connect TCP stream at {address}"))?;

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
