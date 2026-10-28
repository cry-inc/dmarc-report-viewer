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
    pub autonomous_system: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub hosting: Option<bool>,
    pub isp: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub org: Option<String>,
    pub proxy: Option<bool>,
    pub region_name: Option<String>,
    pub timezone: Option<String>,
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
