use crate::config::Configuration;
use crate::mail::{decode_subject, Mail};
use anyhow::{Context, Result};
use async_imap::imap_proto::Address;
use async_imap::types::Fetch;
use async_imap::Client;
use futures::StreamExt;
use std::collections::HashMap;
use std::net::TcpStream as StdTcpStream;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;
use tracing::{debug, info, warn};

pub async fn get_mails(config: &Configuration) -> Result<HashMap<u32, Mail>> {
    let client = create_client(config)
        .await
        .context("Failed to create IMAP client")?;

    let mut session = client
        .login(&config.imap_user, &config.imap_password)
        .await
        .map_err(|e| e.0)
        .context("Failed to log in and create IMAP session")?;
    debug!("IMAP login successful");

    let imap_folder = &config.imap_folder;
    let mailbox = session
        .select(imap_folder)
        .await
        .context(format!("Failed to select {imap_folder} folder"))?;
    debug!("Selected {imap_folder} folder successfully");

    // Get metadata for all all mails and filter by size
    let mut mails = HashMap::new();
    let mut size_filtered_uids = Vec::new();
    debug!(
        "Number of mails in {imap_folder} folder: {}",
        mailbox.exists
    );
    if mailbox.exists > 0 {
        let sequence = format!("1:{}", mailbox.exists);
        let mut stream = session
            .fetch(sequence, "(RFC822.SIZE UID ENVELOPE INTERNALDATE)")
            .await
            .context("Failed to fetch message stream from IMAP inbox")?;
        while let Some(fetch_result) = stream.next().await {
            let fetched =
                fetch_result.context("Failed to get next mail header from IMAP fetch response")?;
            let mail = extract_metadata(&fetched, config.max_mail_size as usize)
                .context("Unable to extract mail metadata")?;
            if mail.oversized {
                // Add oversized mails without body to result list
                mails.insert(mail.uid, mail);
            } else {
                // Get mails with body in next step
                size_filtered_uids.push(mail.uid.to_string());
            }
        }
        if !mails.is_empty() {
            warn!(
                "Found {} mails over size limit of {} bytes",
                mails.len(),
                config.max_mail_size
            )
        }
        info!("Downloaded metadata of {} mails", mailbox.exists)
    }

    // Get full mails for all selected UIDs
    if !size_filtered_uids.is_empty() {
        let mut downloaded = 0;

        // We need to get the mails in chunks.
        // It will fail silently if the requested sequences become too big!
        const CHUNK_SIZE: usize = 5000;
        for chunk in size_filtered_uids.chunks(CHUNK_SIZE) {
            let sequence: String = chunk.join(",");
            let mut stream = session
                .uid_fetch(sequence, "(RFC822 RFC822.SIZE UID ENVELOPE INTERNALDATE)")
                .await
                .context("Failed to fetch message stream from IMAP inbox")?;
            while let Some(fetch_result) = stream.next().await {
                let fetched = fetch_result
                    .context("Failed to get next mail header from IMAP fetch response")?;
                let mut mail = extract_metadata(&fetched, config.max_mail_size as usize)
                    .context("Unable to extract mail metadata")?;
                if let Some(body) = fetched.body() {
                    mail.body = Some(body.to_vec());
                    mail.size = body.len();
                    mails.insert(mail.uid, mail);
                    downloaded += 1;
                } else {
                    warn!("Mail with UID {} has no body!", mail.uid);
                }
            }
        }
        info!("Downloaded {downloaded} mails")
    }

    session
        .logout()
        .await
        .context("Failed to log off from IMAP server")?;

    Ok(mails)
}

/// Creates an encrypted IMAP client
async fn create_client(config: &Configuration) -> Result<Client<TlsStream<TcpStream>>> {
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

    Ok(client)
}

fn extract_metadata(mail: &Fetch, max_size: usize) -> Result<Mail> {
    let uid = mail.uid.context("Mail server did not provide UID")?;
    let size = mail.size.context("Mail server did not provide size")? as usize;
    let env = mail
        .envelope()
        .context("Mail server did not provide envelope")?;
    let sender = addrs_to_string(env.sender.as_deref());
    let to = addrs_to_string(env.to.as_deref());
    let date = mail
        .internal_date()
        .context("Mail server did not provide date")?
        .timestamp();
    let env = mail
        .envelope()
        .context("Mail server did not provide envelope")?;
    let subject = decode_subject(
        env.subject
            .as_deref()
            .map(|s| String::from_utf8_lossy(s))
            .unwrap_or("n/a".into())
            .to_string(),
    );
    Ok(Mail {
        body: None,
        uid,
        sender,
        to,
        subject,
        date,
        size,
        oversized: size > max_size,
        xml_file_count: 0,
    })
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
