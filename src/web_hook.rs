use crate::config::Configuration;
use crate::http_client::http_request;
use crate::state::AppState;
use anyhow::{Context, Result};
use hyper::Method;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

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
    let mut headers: HashMap<String, String> = HashMap::new();
    if let Some(json) = &config.mail_web_hook_headers {
        headers = serde_json::from_str(json).context("Failed to parse optional header JSON")?;
    }

    // Log details of hook call
    debug!("Calling web hook for new mail {mail_id} on URL {url} with method {method}...");

    // Prepare request body
    let body = if let Some(body_str) = &config.mail_web_hook_body {
        let body_str = inject_mail_details(&mail_details, body_str, false)
            .context("Fauled to inject templates into mail body")?;
        body_str.as_bytes().to_vec()
    } else {
        Vec::new()
    };

    // Send HTTP request
    let (status, _, body) = http_request(method, &url, &headers, body)
        .await
        .context("Failed to send HTTP request")?;

    // Check response
    let status_code = status.as_u16();
    debug!("Web hook for new mail {mail_id} responded with status code {status_code}");

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
