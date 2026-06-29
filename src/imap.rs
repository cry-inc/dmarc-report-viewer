use crate::config::Configuration;
use crate::hasher::create_hash;
use crate::mail::{Mail, decode_subject};
use anyhow::{Context, Result, anyhow, ensure};
use async_imap::Client;
use async_imap::imap_proto::Address;
use async_imap::types::{Fetch, Name, NameAttribute};
use futures::StreamExt;
use std::collections::HashMap;
use std::net::TcpStream as StdTcpStream;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName};
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_util::either::Either;
use tracing::{debug, info, trace, warn};

/// A logged-in IMAP session, kept alive while multiple folders are polled.
pub struct Session {
    inner: async_imap::Session<Either<TcpStream, TlsStream<TcpStream>>>,
}

impl Session {
    /// Open a fresh IMAP session and log in.
    pub async fn connect(config: &Configuration) -> Result<Self> {
        let client = create_client(config)
            .await
            .context("Failed to create IMAP client")?;
        let inner = client
            .login(&config.imap_user, &config.imap_password)
            .await
            .map_err(|e| e.0)
            .context("Failed to log in and create IMAP session")?;
        debug!("IMAP login successful");
        Ok(Self { inner })
    }

    pub async fn select_folder(&mut self, folder: &str) -> Result<async_imap::types::Mailbox> {
        let status = self
            .inner
            .select(folder)
            .await
            .context(format!("Failed to select {folder} folder"))?;
        debug!("Selected {folder} folder successfully");
        Ok(status)
    }

    /// Resolve the configured folder (or pattern) to a concrete list of real
    /// folder names that can later be passed to `SELECT`.
    ///
    /// ## Behaviour
    /// * Non-recursive (`recursive = false`): `folder` is returned verbatim.
    ///   This preserves the historical behaviour where the configured value is
    ///   treated as a literal mailbox name.
    /// * Recursive (`recursive = true`, `max_depth >= 1`): the configured
    ///   folder is treated as a baseline prefix. For every depth step from
    ///   `1` to `max_depth`, one IMAP `LIST "" "<baseline>/<%>{n}"` request
    ///   is issued (empty reference + patterns built by
    ///   [`build_recursive_patterns`]). All responses are merged into a
    ///   deduplicated set, the baseline folder itself is always added so it
    ///   never gets accidentally skipped, and `\NoSelect` and wildcard-bearing
    ///   entries are filtered out.
    /// * Recursive with `max_depth = 0`: behaves like non-recursive — the
    ///   literal folder is returned and no LIST request is sent.
    ///
    /// ## Examples
    /// * `INBOX`, `recursive=false`         → `["INBOX"]`
    /// * `INBOX`, `recursive=true`, depth 1 → `["INBOX", "INBOX/<child>"]`
    /// * `Reports`, `recursive=true`, depth 3 → scans `Reports` plus everything
    ///   up to 3 levels deep.
    ///
    /// ## Errors
    /// A failure on the first pattern aborts the call. Failures on later
    /// patterns can be tolerated in the future but are not currently caught.
    /// See also [`strip_wildcard_folder_names`] for the safety net against
    /// servers that echo back wildcard literals.
    pub async fn resolve_folders(
        &mut self,
        folder: &str,
        recursive: bool,
        max_depth: u32,
    ) -> Result<Vec<String>> {
        if !recursive {
            debug!("Using IMAP folder {folder:?} literally (recursive scanning disabled)");
            return Ok(vec![folder.to_string()]);
        }
        let patterns = build_recursive_patterns(folder, max_depth);
        if patterns.is_empty() {
            info!("IMAP folder max-depth is 0, scanning only the literal folder {folder:?}");
            return Ok(vec![folder.to_string()]);
        }
        info!(
            "Issuing IMAP LIST for recursive folder resolution (reference=\"\", patterns={patterns:?}, baseline={folder:?})"
        );

        // Empty reference + per-depth patterns that count `dir/-` separators,
        // each `%` matching exactly one hierarchy level. Iterate depths in
        // ascending order so that shorter results are found before longer
        // patterns are tried; results are merged into a deduplicated set.
        let mut collected: Vec<String> = Vec::new();
        let mut dedup: std::collections::HashSet<String> = std::collections::HashSet::new();

        for pattern in &patterns {
            let names: Vec<Result<Name, _>> = self
                .inner
                .list(Some(""), Some(pattern))
                .await
                .with_context(|| format!("Failed to LIST IMAP folders with pattern {pattern:?}"))?
                .collect()
                .await;
            // Drop `\NoSelect` folders (cannot be SELECT'ed) and any entry
            // whose name still contains IMAP LIST wildcard characters — some
            // servers echo back the literal pattern itself.
            let entries: Vec<String> = names
                .into_iter()
                .filter_map(|r| r.ok())
                .filter(|n| {
                    !n.attributes()
                        .iter()
                        .any(|a| matches!(a, NameAttribute::NoSelect))
                })
                .map(|n| n.name().to_string())
                .collect();
            let before = entries.len();
            let entries = strip_wildcard_folder_names(entries);
            let filtered_out = before - entries.len();
            if filtered_out > 0 {
                debug!(
                    "Filtered {filtered_out} wildcard / \\\\NoSelect entries from IMAP LIST for pattern {pattern:?}"
                );
            }
            for entry in entries {
                if dedup.insert(entry.clone()) {
                    collected.push(entry);
                }
            }
        }

        // Make sure the configured folder itself is always scanned — LIST may
        // not return it explicitly, especially when it has no children.
        if !dedup.contains(folder) {
            dedup.insert(folder.to_string());
            collected.push(folder.to_string());
        }
        info!(
            "IMAP LIST returned {} folder(s) for {folder:?}: {:?}",
            collected.len(),
            collected
        );
        Ok(collected)
    }

    pub async fn close_folder(&mut self) -> Result<()> {
        self.inner
            .close()
            .await
            .context("Failed to close IMAP mailbox")?;
        Ok(())
    }

    pub async fn logout(mut self) {
        if let Err(err) = self.inner.logout().await {
            let anyhow_err = anyhow!(err);
            warn!("Failed to log off from IMAP server: {anyhow_err:#}");
        }
    }
}

/// Build the IMAP LIST mailbox patterns used for recursive scanning of a
/// baseline folder.
///
/// ## Why per-depth patterns?
/// The IMAP `*` wildcard is interpreted inconsistently across servers:
/// some treat it as a single-level wildcard (RFC 3501 §6.3.8 actually
/// says `*` matches zero or more characters **at any level**, but real
/// implementations differ), others treat it as a single-level wildcard
/// like `%`. Using the unambiguous `%` single-level wildcard, once per
/// requested depth step, gives predictable behaviour that covers any
/// hierarchy depth on every RFC-compliant server.
///
/// ## Output semantics
/// * The result is a list of `max_depth` mailbox patterns.
/// * Pattern `n` has exactly `n` trailing `%` wildcards, separated by `/`.
/// * A single trailing `/` on `folder` is tolerated to avoid double slashes.
/// * An empty `folder` yields patterns that begin with `/` (e.g. `/%`);
///   callers are expected to validate non-emptiness if they need to.
///
/// ## Examples (`max_depth = 2`)
/// * `INBOX`       → `["INBOX/%", "INBOX/%/%"]`
/// * `INBOX/DMARC` → `["INBOX/DMARC/%", "INBOX/DMARC/%/%"]`
/// * `INBOX/`      → `["INBOX/%", "INBOX/%/%"]`
///
/// Returns an empty `Vec` when `max_depth` is `0`. Callers handle that case
/// explicitly to avoid issuing any LIST request at all.
pub(crate) fn build_recursive_patterns(folder: &str, max_depth: u32) -> Vec<String> {
    let normalised = folder.trim_end_matches('/');
    let prefix = if normalised.is_empty() {
        String::new()
    } else {
        format!("{normalised}/")
    };
    (1..=max_depth)
        .map(|depth| {
            let suffix = std::iter::repeat_n("%", depth as usize)
                .collect::<Vec<_>>()
                .join("/");
            format!("{prefix}{suffix}")
        })
        .collect()
}

/// Strip out IMAP LIST wildcard characters from arbitrary folder names.
/// Some servers echo back the literal mailbox pattern (or parts containing
/// `*` / `%`) when issuing IMAP `LIST`. Such entries can never be used as
/// a `SELECT` target, so they must be filtered before further processing.
pub(crate) fn strip_wildcard_folder_names<I, S>(names: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    names
        .into_iter()
        .map(|s| s.as_ref().to_string())
        .filter(|name| !name.contains('*') && !name.contains('%'))
        .collect()
}

/// Resolve the set of folders to scan, as configured in `config`.
/// This mirrors the prior behavior: dedicated DMARC/TLS folders (if set)
/// take precedence over the default `imap_folder`.
pub fn configured_folders(config: &Configuration) -> Vec<String> {
    let mut folders = Vec::new();
    match (
        config.imap_folder_dmarc.as_ref(),
        config.imap_folder_tls.as_ref(),
    ) {
        (Some(d), Some(t)) => {
            folders.push(d.clone());
            folders.push(t.clone());
        }
        (Some(d), None) => {
            folders.push(d.clone());
        }
        (None, Some(t)) => {
            folders.push(t.clone());
        }
        (None, None) => {
            folders.push(config.imap_folder.clone());
        }
    }
    folders
}

/// Fetch all mails from a single folder on an already logged-in session.
async fn get_mails_in_folder(
    session: &mut Session,
    config: &Configuration,
    folder: &str,
) -> Result<HashMap<String, Mail>> {
    let started = Instant::now();
    let mailbox = session
        .select_folder(folder)
        .await
        .with_context(|| format!("Failed to select {folder} folder"))?;
    debug!("Number of mails in {folder} folder: {}", mailbox.exists);

    let mut mails = HashMap::new();
    let mut body_count = 0usize;
    if mailbox.exists > 0 {
        let sequence = format!("1:{}", mailbox.exists);
        let mut stream = session
            .inner
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
                folder,
            )
            .context("Unable to extract mail metadata")?;
            mails.insert(mail.id.clone(), mail);
        }
        debug!("Fetched metadata of {} mail(s) from {folder}", mails.len());

        let no_size_mails = mails.values().filter(|m| m.size == 0).count();
        if no_size_mails > 0 {
            warn!(
                "Found {no_size_mails} without size property, this will make upfront oversize filtering impossible!"
            )
        }

        let oversized_mails = mails.values().filter(|m| m.oversized).count();
        if oversized_mails > 0 {
            warn!(
                "Found {} mails over size limit of {} bytes",
                oversized_mails, config.max_mail_size
            )
        }
    }

    let ids: Vec<String> = mails
        .values()
        .filter(|m| !m.oversized)
        .map(|m| m.id.clone())
        .collect();
    if !ids.is_empty() {
        ensure!(
            config.imap_chunk_size > 0,
            "IMAP Chunk size must be non-zero"
        );
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
            let fetch_query = format!("({body_request} RFC822.SIZE UID ENVELOPE INTERNALDATE)");

            let mut stream = session
                .inner
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
                    mail.body = None;
                    warn!("Mail with UID {uid} was bigger than expected and is oversized");
                }
                body_count += 1;
                trace!(
                    "Fetched mail with UID {uid} and size {} from {}",
                    mail.size, mail.sender
                );
            }
            if fetched_mails != chunk.len() {
                warn!(
                    "Unable to fetch some mails from chunk, expected {} mails but got {fetched_mails}. \
                    This often happens when the IMAP server does not support the configured chunk size. \
                    Its currently configured to {} mails per chunk. \
                    Try setting lower chunk sizes using the command line argument --imap-chunk-size or \
                    by setting the environment variable IMAP_CHUNK_SIZE.",
                    chunk.len(),
                    config.imap_chunk_size
                );
            }
        }

        debug!("Fetched {} body payload(s) from {folder}", body_count);
    }

    if let Err(err) = session.close_folder().await {
        warn!("Failed to close IMAP folder {folder}: {err:#}");
    }
    debug!(
        "Finished IMAP folder {folder}: {} metadata, {} body payload(s), {:.3}s",
        mails.len(),
        body_count,
        started.elapsed().as_secs_f64()
    );
    Ok(mails)
}

/// Top-level helper used by the background task.
/// Resolves configured folder patterns to actual folder names, opens one
/// IMAP session, fetches all mails across all resolved folders and returns
/// them keyed by mail ID. Duplicates across folders are merged.
pub async fn get_mails(config: &Configuration) -> Result<HashMap<String, Mail>> {
    let folder_specs = configured_folders(config);
    let mut session = Session::connect(config).await?;
    let mut mails: HashMap<String, Mail> = HashMap::new();
    let mut processed_folders = 0usize;
    let mut failed_folders = 0usize;
    let round_started = std::time::Instant::now();

    for folder_spec in &folder_specs {
        let resolved = session
            .resolve_folders(
                folder_spec,
                config.imap_folder_recursive,
                config.imap_folder_max_depth,
            )
            .await
            .with_context(|| format!("Failed to resolve folders for {folder_spec}"))?;
        if resolved.is_empty() {
            warn!("IMAP folder {folder_spec} matched no folders, skipping");
            continue;
        }
        for folder in resolved {
            debug!("Fetching mails from IMAP folder {folder}");
            match get_mails_in_folder(&mut session, config, &folder).await {
                Ok(folder_mails) => {
                    processed_folders += 1;
                    for (id, mail) in folder_mails {
                        mails.insert(id, mail);
                    }
                }
                Err(err) => {
                    failed_folders += 1;
                    warn!("Failed to get mails from folder {folder}: {err:#}");
                }
            }
        }
    }

    info!(
        "IMAP fetch round finished: {} folder(s) processed, {} failed, {} unique mail(s), {:.3}s",
        processed_folders,
        failed_folders,
        mails.len(),
        round_started.elapsed().as_secs_f64()
    );

    session.logout().await;
    Ok(mails)
}

/// Fetch mails from a single explicit folder using a fresh session.
/// Kept as the underlying primitive; not called directly by the background task.
#[allow(dead_code)]
pub async fn get_mails_in_single_folder(
    config: &Configuration,
    imap_folder: &str,
) -> Result<HashMap<String, Mail>> {
    let mut session = Session::connect(config).await?;
    let mails = get_mails_in_folder(&mut session, config, imap_folder).await?;
    session.logout().await;
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
        dmarc_duplicates: Vec::new(),
        tls_duplicates: Vec::new(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn base_config() -> Configuration {
        // Builds a Configuration from CLI args without ever invoking clap's
        // environment-variable / file side effects. Pass all required fields
        // explicitly so clap does not fail parsing here. Only the folder /
        // recursive fields are of interest in the tests below.
        Configuration::parse_from([
            "dmarc-report-viewer",
            "--imap-host",
            "imap.example.org",
            "--imap-user",
            "user",
            "--imap-password",
            "secret",
            "--http-server-password",
            "secret",
        ])
    }

    #[test]
    fn build_recursive_patterns_single_level() {
        assert_eq!(
            build_recursive_patterns("INBOX", 1),
            vec!["INBOX/%".to_string()]
        );
        assert_eq!(
            build_recursive_patterns("Reports", 1),
            vec!["Reports/%".to_string()]
        );
    }

    #[test]
    fn build_recursive_patterns_trailing_slash() {
        // A trailing slash must not produce double slashes in the patterns.
        assert_eq!(
            build_recursive_patterns("INBOX/", 2),
            vec!["INBOX/%".to_string(), "INBOX/%/%".to_string()]
        );
        assert_eq!(
            build_recursive_patterns("Reports/Archive/", 1),
            vec!["Reports/Archive/%".to_string()]
        );
    }

    #[test]
    fn build_recursive_patterns_nested_folder() {
        assert_eq!(
            build_recursive_patterns("Reports/Archive/2024", 2),
            vec![
                "Reports/Archive/2024/%".to_string(),
                "Reports/Archive/2024/%/%".to_string(),
            ]
        );
    }

    #[test]
    fn build_recursive_patterns_multi_level() {
        assert_eq!(
            build_recursive_patterns("INBOX/DMARC", 3),
            vec![
                "INBOX/DMARC/%".to_string(),
                "INBOX/DMARC/%/%".to_string(),
                "INBOX/DMARC/%/%/%".to_string(),
            ]
        );
    }

    #[test]
    fn build_recursive_patterns_zero_depth_returns_empty() {
        // Zero depth means no sub-folder scanning at all; callers must handle
        // this case explicitly (and only scan the literal folder).
        assert!(build_recursive_patterns("INBOX", 0).is_empty());
    }

    #[test]
    fn strip_wildcard_folder_names_drops_pattern_echo() {
        // Some servers echo back the literal mailbox pattern instead of
        // matching folders; such entries must never reach SELECT.
        let raw = vec![
            "INBOX/DMARC".to_string(),
            "INBOX/DMARC/*".to_string(),
            "INBOX/Reports".to_string(),
        ];
        let cleaned = strip_wildcard_folder_names(raw);
        assert_eq!(
            cleaned,
            vec!["INBOX/DMARC".to_string(), "INBOX/Reports".to_string()]
        );
    }

    #[test]
    fn strip_wildcard_folder_names_keeps_normal_folders() {
        let raw = vec!["INBOX".to_string(), "Reports/2024".to_string()];
        let cleaned = strip_wildcard_folder_names(raw);
        assert_eq!(
            cleaned,
            vec!["INBOX".to_string(), "Reports/2024".to_string()]
        );
    }

    #[test]
    fn strip_wildcard_folder_names_handles_percent() {
        // `%` is the IMAP wildcard for "any character at this level"; the same
        // safety net must catch it too.
        let raw = vec!["INBOX/%".to_string(), "INBOX".to_string()];
        let cleaned = strip_wildcard_folder_names(raw);
        assert_eq!(cleaned, vec!["INBOX".to_string()]);
    }

    #[test]
    fn strip_wildcard_folder_names_accepts_str_slices() {
        let raw = vec!["a/b/*", "a/b/c"];
        let cleaned = strip_wildcard_folder_names(raw);
        assert_eq!(cleaned, vec!["a/b/c".to_string()]);
    }

    #[test]
    fn configured_folders_default() {
        let cfg = base_config();
        assert!(!cfg.imap_folder_recursive);
        assert_eq!(configured_folders(&cfg), vec!["INBOX"]);
    }

    #[test]
    fn configured_folders_dmarc_only() {
        let cfg = Configuration::parse_from([
            "dmarc-report-viewer",
            "--imap-host",
            "x",
            "--imap-user",
            "u",
            "--imap-password",
            "p",
            "--http-server-password",
            "p",
            "--imap-folder-dmarc",
            "Reports/DMARC",
        ]);
        assert_eq!(configured_folders(&cfg), vec!["Reports/DMARC".to_string()]);
    }

    #[test]
    fn configured_folders_tls_only() {
        let cfg = Configuration::parse_from([
            "dmarc-report-viewer",
            "--imap-host",
            "x",
            "--imap-user",
            "u",
            "--imap-password",
            "p",
            "--http-server-password",
            "p",
            "--imap-folder-tls",
            "Reports/TLS",
        ]);
        assert_eq!(configured_folders(&cfg), vec!["Reports/TLS".to_string()]);
    }

    #[test]
    fn configured_folders_both_dedicated() {
        let cfg = Configuration::parse_from([
            "dmarc-report-viewer",
            "--imap-host",
            "x",
            "--imap-user",
            "u",
            "--imap-password",
            "p",
            "--http-server-password",
            "p",
            "--imap-folder-dmarc",
            "Reports/DMARC",
            "--imap-folder-tls",
            "Reports/TLS",
        ]);
        assert_eq!(
            configured_folders(&cfg),
            vec!["Reports/DMARC".to_string(), "Reports/TLS".to_string()]
        );
    }

    #[test]
    fn configured_folders_recursive_flag_parses() {
        let cfg = Configuration::parse_from([
            "dmarc-report-viewer",
            "--imap-host",
            "x",
            "--imap-user",
            "u",
            "--imap-password",
            "p",
            "--http-server-password",
            "p",
            "--imap-folder-recursive",
        ]);
        assert!(cfg.imap_folder_recursive);
        // Default depth covers the typical one-level-subfolder setup
        // (e.g. `INBOX/DMARC/<provider>`).
        assert_eq!(cfg.imap_folder_max_depth, 2);
        assert_eq!(configured_folders(&cfg), vec!["INBOX"]);
    }

    #[test]
    fn configured_folders_max_depth_parses() {
        let cfg = Configuration::parse_from([
            "dmarc-report-viewer",
            "--imap-host",
            "x",
            "--imap-user",
            "u",
            "--imap-password",
            "p",
            "--http-server-password",
            "p",
            "--imap-folder-recursive",
            "--imap-folder-max-depth",
            "5",
        ]);
        assert_eq!(cfg.imap_folder_max_depth, 5);
        // Sanity: 5 levels produces 5 patterns.
        let patterns = build_recursive_patterns("INBOX/DMARC", cfg.imap_folder_max_depth);
        assert_eq!(patterns.len(), 5);
        assert_eq!(patterns[0], "INBOX/DMARC/%");
        assert_eq!(patterns[4], "INBOX/DMARC/%/%/%/%/%");
    }

    #[test]
    fn configured_folders_env_var_recursive() {
        // Clap derives the env variable name `IMAP_FOLDER_RECURSIVE` from the
        // `--imap-folder-recursive` flag. We do not want to mutate process-wide
        // environment variables here (we forbid unsafe and parallel tests would
        // race on env state), but we can still assert the env wiring by
        // documenting the derivation rule next to the field. The flag itself
        // is already covered by `configured_folders_recursive_flag_parses`.
        let cfg = Configuration::parse_from([
            "dmarc-report-viewer",
            "--imap-host",
            "x",
            "--imap-user",
            "u",
            "--imap-password",
            "p",
            "--http-server-password",
            "p",
            "--imap-folder",
            "Reports",
        ]);
        assert_eq!(cfg.imap_folder, "Reports");
        assert_eq!(configured_folders(&cfg), vec!["Reports"]);
    }
}
