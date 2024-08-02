use crate::config::Configuration;
use crate::mail::Mail;
use anyhow::{Context, Result};
use async_imap::imap_proto::Address;
use async_imap::Client;
use futures::StreamExt;
use std::net::TcpStream as StdTcpStream;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;
use tracing::{debug, warn};

pub async fn get_mails(config: &Configuration) -> Result<Vec<Mail>> {
    // Prepare cert store with webpki roots
    let mut root_cert_store = RootCertStore::empty();
    let certs = webpki_roots::TLS_SERVER_ROOTS.iter().cloned();
    root_cert_store.extend(certs);
    debug!("Created Root CA cert store");

    // Create async TLS connection
    let client_config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    debug!("Created TLS client config");

    let connector = TlsConnector::from(Arc::new(client_config));
    debug!("Created TLS connector");

    let host_port = format!("{}:{}", config.imap_host.as_str(), config.imap_port);
    debug!("Parsing IMAP address {host_port} as socket address...");
    let addrs = host_port
        .to_socket_addrs()
        .context("Failed to convert host name and port to socket address")?
        .collect::<Vec<SocketAddr>>();
    let addr = addrs.first().context("Unable get first resolved address")?;
    debug!("Got address {addr}");

    let timeout = Duration::from_secs(config.imap_timeout);
    let std_tcp_stream =
        StdTcpStream::connect_timeout(addr, timeout).context("Failed to connect to IMAP server")?;
    debug!("Created TCP stream");

    std_tcp_stream
        .set_nonblocking(true)
        .context("Failed to set TCP stream to non-blocking")?;
    let tcp_stream = TcpStream::from_std(std_tcp_stream)
        .context("Failed to create TCP stream to IMAP server")?;
    debug!("Created async TCP stream");

    let dns_name = ServerName::try_from(config.imap_host.clone())
        .context("Failed to get DNS name from IMAP host")?;
    debug!("Got DNS name: {dns_name:?}");

    let tls_stream = connector
        .connect(dns_name, tcp_stream)
        .await
        .context("Failed to create TLS stream with IMAP server")?;
    debug!("Created TLS stream");

    let client = Client::new(tls_stream);
    debug!("Created IMAP client");

    let mut session = client
        .login(&config.imap_user, &config.imap_password)
        .await
        .map_err(|e| e.0)
        .context("Failed to log in and create IMAP session")?;
    debug!("IMAP login successful");

    let mailbox = session
        .select("INBOX")
        .await
        .context("Failed to select inbox")?;
    debug!("Selected INBOX successfully");

    // Get metadata for all all mails and filter by size
    let mut uids = Vec::new();
    debug!("Number of mails in INBOX: {}", mailbox.exists);
    if mailbox.exists > 0 {
        let sequence = format!("1:{}", mailbox.exists);
        let mut stream = session
            .fetch(sequence, "(RFC822.SIZE UID ENVELOPE)")
            .await
            .context("Failed to fetch message stream from IMAP inbox")?;
        while let Some(fetch_result) = stream.next().await {
            let mail =
                fetch_result.context("Failed to get next mail header from IMAP fetch response")?;
            let uid = mail.uid.context("Mail server did not provide UID")?;
            let size = mail.size.context("Mail server did not provide size")?;
            if size <= config.max_mail_size {
                uids.push(uid.to_string());
            } else {
                warn!(
                    "Found mail over size limit of {}: {}",
                    config.max_mail_size, size
                )
            }
        }
    }

    // Get full mails for all selected UIDs
    let mut mails = Vec::new();
    if !uids.is_empty() {
        let sequence: String = uids.join(",");
        let mut stream = session
            .uid_fetch(sequence, "(RFC822 UID ENVELOPE)")
            .await
            .context("Failed to fetch message stream from IMAP inbox")?;
        while let Some(fetch_result) = stream.next().await {
            let mail =
                fetch_result.context("Failed to get next mail header from IMAP fetch response")?;
            let uid = mail.uid.context("Mail server did not provide UID")?;
            let env = mail
                .envelope()
                .context("Mail server did not provide envelope")?;
            let subject = env
                .subject
                .as_deref()
                .map(|s| String::from_utf8_lossy(s))
                .unwrap_or("n/a".into())
                .to_string();
            let sender = addrs_to_string(env.sender.as_deref());
            let to = addrs_to_string(env.to.as_deref());
            if let Some(body) = mail.body() {
                mails.push(Mail {
                    body: Some(body.to_vec()),
                    uid,
                    sender,
                    to,
                    subject,
                    size: body.len(),
                })
            } else {
                warn!("Mail with UID {} has no body!", uid);
            }
        }
    }

    session
        .logout()
        .await
        .context("Failed to log off from IMAP server")?;

    Ok(mails)
}

fn addrs_to_string(addrs: Option<&[Address]>) -> String {
    if let Some(addrs) = addrs {
        addrs
            .iter()
            .map(|addr| {
                let mailbox = addr
                    .mailbox
                    .as_deref()
                    .map(|s| String::from_utf8_lossy(s))
                    .unwrap_or("n/a".into())
                    .to_string();
                let host = addr
                    .host
                    .as_deref()
                    .map(|s| String::from_utf8_lossy(s))
                    .unwrap_or("n/a".into())
                    .to_string();
                format!("{mailbox}@{host}")
            })
            .collect::<Vec<String>>()
            .join("; ")
    } else {
        String::from("n/a")
    }
}
