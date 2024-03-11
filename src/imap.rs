use crate::config::Configuration;
use anyhow::{Context, Result};
use async_imap::Client;
use futures::StreamExt;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;
use tracing::warn;

pub async fn get_mails(config: &Configuration) -> Result<Vec<Vec<u8>>> {
    // Prepare cert store with webpki roots
    let mut root_cert_store = RootCertStore::empty();
    let certs = webpki_roots::TLS_SERVER_ROOTS.iter().cloned();
    root_cert_store.extend(certs);

    // Create async TLS connection
    let client_config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(client_config));
    let dns_name = ServerName::try_from(config.imap_host.clone())
        .context("Failed to get DNS name from IMAP host")?;
    let addr = (config.imap_host.as_str(), config.imap_port);
    let tcp_stream = TcpStream::connect(&addr)
        .await
        .context("Failed to create TCP stream to IMAP server")?;
    let tls_stream = connector
        .connect(dns_name, tcp_stream)
        .await
        .context("Failed to create TLS stream with IMAP server")?;

    let client = Client::new(tls_stream);
    let mut session = client
        .login(&config.imap_user, &config.imap_password)
        .await
        .map_err(|e| e.0)
        .context("Failed to log in and create IMAP session")?;
    let mailbox = session
        .select("INBOX")
        .await
        .context("Failed to select inbox")?;
    let mut mails = Vec::new();
    if mailbox.exists > 0 {
        let sequence = format!("1:{}", mailbox.exists);
        let mut message_stream = session
            .fetch(sequence, "RFC822")
            .await
            .context("Failed to fetch message stream from IMAP inbox")?;
        while let Some(message) = message_stream.next().await {
            let message = message.context("Failed to get next message from IMAP inbox")?;
            match message.body() {
                Some(body) => mails.push(body.to_vec()),
                None => warn!("Found a message without a body!"),
            }
        }
    }
    session
        .logout()
        .await
        .context("Failed to log off from IMAP server")?;
    Ok(mails)
}
