use crate::config::Configuration;
use crate::hasher::create_hash;
use crate::mail::{decode_subject, Mail};
use anyhow::{anyhow, Context, Result};
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
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName};
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;
use tokio_util::either::Either;
use tracing::{debug, info, trace, warn};

pub async fn get_mails(config: &Configuration) -> Result<HashMap<String, Mail>> {
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
    debug!(
        "Number of mails in {imap_folder} folder: {}",
        mailbox.exists
    );
    if mailbox.exists > 0 {
        // Get metadata for all mails
        let sequence = format!("1:{}", mailbox.exists);
        let mut stream = session
            .fetch(sequence, "(RFC822.SIZE UID ENVELOPE INTERNALDATE)")
            .await
            .context("Failed to fetch message stream from IMAP inbox")?;
        while let Some(fetch_result) = stream.next().await {
            let fetched =
                fetch_result.context("Failed to get next mail header from IMAP fetch response")?;
            let mail = extract_metadata(
                &fetched,
                config.max_mail_size as usize,
                &config.imap_user,
                &config.imap_folder,
            )
            .context("Unable to extract mail metadata")?;
            mails.insert(mail.id.clone(), mail);
        }
        info!("Downloaded metadata of {} mails", mails.len());

        let no_size_mails = mails.values().filter(|m| m.size == 0).count();
        if no_size_mails > 0 {
            warn!("Found {no_size_mails} without size property, this will make upfront oversize filtering impossible!")
        }

        let oversized_mails = mails.values().filter(|m| m.oversized).count();
        if oversized_mails > 0 {
            warn!(
                "Found {} mails over size limit of {} bytes",
                oversized_mails, config.max_mail_size
            )
        }
    }

    // Get full mail body for all non-oversized mails
    let ids: Vec<String> = mails
        .values()
        .filter(|m| !m.oversized)
        .map(|m| m.id.clone())
        .collect();
    if !ids.is_empty() {
        // We need to get the mails in chunks.
        // It will fail silently if the requested sequences become too big!
        for chunk in ids.chunks(config.imap_chunk_size) {
            debug!("Downloading chunk with {} mails...", chunk.len());
            let mut uid_id_map = HashMap::new();
            let uids: Vec<String> = chunk
                .iter()
                .map(|id| {
                    let me = &mails[id];
                    uid_id_map.insert(me.uid, me.id.clone());
                    me.uid
                })
                .map(|uid| uid.to_string())
                .collect();
            let uid_sequence: String = uids.join(",");
            let body_request = config.imap_body_request.to_request_string();

            // Some servers (like iCloud Mail) seem to require BODY[] instead of just RFC822...
            let fetch_query = format!("({body_request} RFC822.SIZE UID ENVELOPE INTERNALDATE)");

            let mut stream = session
                .uid_fetch(uid_sequence, &fetch_query)
                .await
                .context("Failed to fetch message stream from IMAP inbox")?;
            let mut fetched_mails = 0;
            while let Some(fetch_result) = stream.next().await {
                let fetched = fetch_result
                    .context("Failed to get next mail header from IMAP fetch response")?;
                fetched_mails += 1;
                let uid = fetched
                    .uid
                    .context("Failed to get UID from IMAP fetch result")?;
                let Some(id) = uid_id_map.get(&uid) else {
                    warn!("Cannot find existing mail ID for UID {uid}");
                    continue;
                };
                let Some(mail) = mails.get_mut(id) else {
                    warn!("Cannot find mail entry for ID {id}");
                    continue;
                };
                let Some(body) = fetched.body() else {
                    warn!("Mail with UID {uid} has no body!");
                    continue;
                };
                mail.body = Some(body.to_vec());
                mail.size = body.len();
                mail.oversized = body.len() > config.max_mail_size as usize;
                if mail.oversized {
                    // Do not keep oversized mails in memory
                    mail.body = None;
                    warn!("Mail with UID {uid} was bigger than expected and is oversized");
                }
                trace!(
                    "Fetched mail with UID {uid} and size {} from {}",
                    mail.size,
                    mail.sender
                );
            }
            if fetched_mails != chunk.len() {
                warn!(
                    "Unable to fetch some mails from chunk, expected {} mails but got {fetched_mails}",
                    chunk.len()
                );
            }
        }

        info!("Downloaded {} mails", ids.len());
    }

    // We have everything we need, an error is no longer preventing an update.
    if let Err(err) = session.logout().await {
        let anyhow_err = anyhow!(err);
        warn!("Failed to log off from IMAP server: {anyhow_err:#}");
    }

    Ok(mails)
}

/// Creates an unecrypted or encrypted IMAP client
async fn create_client(
    config: &Configuration,
) -> Result<Client<Either<TcpStream, TlsStream<TcpStream>>>> {
    let host_port = format!("{}:{}", config.imap_host.as_str(), config.imap_port);
    let addrs = host_port
        .to_socket_addrs()
        .context("Failed to convert host name and port to socket address")?
        .collect::<Vec<SocketAddr>>();
    let addr = addrs.first().context("Unable get first resolved address")?;
    debug!("Got {addr} as as socket address for IMAP endpoint {host_port}");

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

    let stream = if config.imap_starttls {
        debug!("Sending STARTTLS command over plain connection...");
        let mut plain_client = Client::new(tcp_stream);
        plain_client
            .read_response()
            .await
            .context("Failed to read greeting")?
            .context("Failed parse greeting response")?;
        debug!("Received greeting");
        plain_client
            .run_command_and_check_ok("STARTTLS", None)
            .await
            .context("Failed to run STARTTLS command")?;
        debug!("Requested STARTTLS, upgrading...");
        let tls_stream = create_tls_stream(config, plain_client.into_inner())
            .await
            .context("Failed to upgrade to TLS stream")?;
        Either::Right(tls_stream)
    } else if config.imap_disable_tls {
        warn!("Using unecrypted TCP connection for IMAP client");
        Either::Left(tcp_stream)
    } else {
        debug!("Directly creating TLS stream...");
        let tls_stream = create_tls_stream(config, tcp_stream)
            .await
            .context("Failed to create TLS stream")?;
        Either::Right(tls_stream)
    };

    let client = Client::new(stream);
    debug!("Created IMAP client");
    Ok(client)
}

async fn create_tls_stream(
    config: &Configuration,
    tcp_stream: TcpStream,
) -> Result<TlsStream<TcpStream>> {
    let mut root_cert_store = RootCertStore::empty();
    let certs = webpki_roots::TLS_SERVER_ROOTS.iter().cloned();
    root_cert_store.extend(certs);
    debug!("Created Root CA cert store");

    if let Some(ca_certs) = &config.imap_tls_ca_certs {
        info!(
            "Loading file with custom TLS CA certificates for IMAP client from {}...",
            ca_certs.display()
        );
        let mut custom_certs = Vec::new();
        for res in CertificateDer::pem_file_iter(ca_certs)
            .context("Failed to parse custom CA certificate file")?
        {
            let cert = res.context("Failed to parse certificate")?;
            custom_certs.push(cert);
        }
        info!(
            "Loaded {} custom certificate(s) from input file",
            custom_certs.len()
        );
        let (added, ignored) = root_cert_store.add_parsable_certificates(custom_certs);
        info!("{added} certificate(s) were added to the root store and {ignored} were ignored");
    }

    let client_config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    debug!("Created TLS client config");

    let connector = TlsConnector::from(Arc::new(client_config));
    debug!("Created TLS connector");

    let dns_name = ServerName::try_from(config.imap_host.clone())
        .context("Failed to get DNS name from host")?;
    debug!("Got DNS name: {dns_name:?}");

    let tls_stream = connector
        .connect(dns_name, tcp_stream)
        .await
        .context("Failed to create TLS stream")?;
    debug!("Created TLS stream");

    Ok(tls_stream)
}

fn extract_metadata(mail: &Fetch, max_size: usize, account: &str, folder: &str) -> Result<Mail> {
    let uid = mail.uid.context("Mail server did not provide UID")?;
    let size = mail.size.unwrap_or(0) as usize; // In case the mail server ignored our request for the size
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
            .as_ref(),
    );

    // The UID is not globally unique, so we need to add some other properties!
    let id = create_hash(&[&uid.to_le_bytes(), account.as_bytes(), folder.as_bytes()]);

    Ok(Mail {
        id,
        account: account.to_string(),
        folder: folder.to_string(),
        body: None,
        uid,
        sender,
        to,
        subject,
        date,
        size,
        oversized: size > max_size,
        xml_files: 0,
        json_files: 0,
        xml_parsing_errors: 0,
        json_parsing_errors: 0,
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
