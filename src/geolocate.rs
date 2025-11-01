use crate::http_client::http_request;
use anyhow::{Context, Result, ensure};
use hyper::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

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
    pub async fn from_ip(ip: &IpAddr) -> Result<Option<Self>> {
        // Create URL
        let url = format!(
            "http://ip-api.com/json/{ip}?fields=country,countryCode,regionName,city,lat,lon,timezone,isp,org,as,proxy,hosting,query"
        );

        // Send HTTP request
        let (status, _, body) = http_request(Method::GET, &url, &HashMap::new(), Vec::new())
            .await
            .context("Failed to send HTTP request")?;
        ensure!(status == StatusCode::OK);

        // Parse response JSON
        let parsed: Self =
            serde_json::from_slice(&body).context("Failed to parse HTTP response as JSON")?;

        Ok(Some(parsed))
    }
}
