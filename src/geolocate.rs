use anyhow::{bail, ensure, Context, Result};
use axum::http::uri::Scheme;
use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper::client::conn::http1;
use hyper::{Request, Uri};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    #[serde(rename = "as")]
    pub autonomous_system: String,
    pub country: String,
    pub city: String,
    pub country_code: String,
    pub hosting: bool,
    pub isp: String,
    pub lat: f64,
    pub lon: f64,
    pub org: String,
    pub proxy: bool,
    pub region_name: String,
    pub timezone: String,
}

impl Location {
    /// Current backend allows 45 requests per minute
    pub async fn from_ip(ip: &str) -> Result<Option<Self>> {
        // Create and parse URI
        let uri = format!("http://ip-api.com/json/{ip}?fields=country,countryCode,regionName,city,lat,lon,timezone,isp,org,as,proxy,hosting,query")
            .parse::<Uri>()
            .context("Failed to parse URI")?;
        ensure!(
            uri.scheme().context("URI has no scheme")? == &Scheme::HTTP,
            "Only HTTP is supported"
        );

        // Get the host and the port
        let host = uri.host().context("URI has no host")?.to_string();
        let port = uri.port_u16().unwrap_or(80);

        // Open a TCP connection to the remote host
        let address = format!("{host}:{port}");
        let stream = TcpStream::connect(address)
            .await
            .context("Failed to connect TCP stream")?;

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

        // Create and send HTTP request
        let req = Request::builder()
            .uri(uri)
            .header(hyper::header::HOST, host)
            .body(Empty::<Bytes>::new())
            .context("Failed to create HTTP request")?;
        let mut res = sender
            .send_request(req)
            .await
            .context("Failed to send HTTP request")?;
        ensure!(res.status().is_success(), "HTTP request did not succeed");

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

        // Parse response JSON
        let parsed: Self =
            serde_json::from_slice(&body).context("Failed to parse HTTP response as JSON")?;

        Ok(Some(parsed))
    }
}
