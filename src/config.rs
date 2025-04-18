use clap::Parser;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::{info, Level};

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct Configuration {
    /// Host name or domain of the IMAP server with the DMARC reports inbox
    #[arg(long, env)]
    pub imap_host: String,

    /// User name of the IMAP inbox with the DMARC reports
    #[arg(long, env)]
    pub imap_user: String,

    /// Password of the IMAP inbox with the DMARC reports
    #[arg(long, env)]
    pub imap_password: String,

    /// TLS encrypted port of the IMAP server
    #[arg(long, env, default_value_t = 993)]
    pub imap_port: u16,

    /// Enable STARTTLS mode for IMAP client (IMAP port should be set to 143)
    #[arg(long, env, conflicts_with = "imap_disable_tls")]
    pub imap_starttls: bool,

    /// Optional path to additional TLS root certificates used to creating the IMAP TLS connections.
    /// The default set is a compiled-in copy of the root certificates trusted by Mozilla.
    /// The path should point to a PEM file with one or more X.509 certificates.
    #[arg(long, env)]
    pub imap_tls_ca_certs: Option<PathBuf>,

    /// Will the disable TLS encryption for the IMAP connection (IMAP port should be set to 143).
    /// Not recommended. NEVER use this for a remote IMAP server over a network!
    /// This is ONLY intended for connecting to IMAP servers or proxies on the same machine!
    #[arg(long, env, conflicts_with = "imap_starttls")]
    pub imap_disable_tls: bool,

    /// IMAP folder with the DMARC reports
    #[arg(long, env, default_value = "INBOX")]
    pub imap_folder: String,

    /// Method of requesting the mail body from the IMAP server.
    /// The default should work for most cases.
    /// Only try other values if you have issues with missing mail bodies.
    /// Possible values: default, rfc822 and body.
    #[arg(long, env, default_value_t = ImapBodyRequest::Default)]
    pub imap_body_request: ImapBodyRequest,

    /// TCP connection timeout for IMAP server in seconds
    #[arg(long, env, default_value_t = 10)]
    pub imap_timeout: u64,

    /// Number of mails downloaded in one chunk, must be bigger than 0.
    #[arg(long, env, default_value_t = 5000)]
    pub imap_chunk_size: usize,

    /// Interval between checking for new reports in IMAP inbox in seconds
    #[arg(long, env, default_value_t = 1800)]
    pub imap_check_interval: u64,

    /// Embedded HTTP server port for web UI
    #[arg(long, env, default_value_t = 8080)]
    pub http_server_port: u16,

    /// Embedded HTTP server binding for web UI
    #[arg(long, env, default_value = "0.0.0.0")]
    pub http_server_binding: String,

    /// Username for the HTTP server basic auth login
    #[arg(long, env, default_value = "dmarc")]
    pub http_server_user: String,

    /// Password for the HTTP server basic auth login.
    /// Use empty string to disable (not recommended).
    #[arg(long, env)]
    pub http_server_password: String,

    /// Enable automatic HTTPS encryption using Let's Encrypt certificates.
    /// This will replace the HTTP protocol on the configured HTTP port with HTTPS.
    /// There is no second separate port for HTTPS!
    /// This uses the TLS-ALPN-01 challenge and therefore the public HTTPS port MUST be 443!
    #[arg(
        long,
        env,
        requires = "https_auto_cert_domain",
        requires = "https_auto_cert_mail",
        requires = "https_auto_cert_cache"
    )]
    pub https_auto_cert: bool,

    /// Contact E-Mail address, required for automatic HTTPS
    #[arg(long, env)]
    pub https_auto_cert_mail: Option<String>,

    /// Certificate caching directory, required for automatic HTTPS
    #[arg(long, env)]
    pub https_auto_cert_cache: Option<PathBuf>,

    /// HTTPS server domain, required for automatic HTTPS
    #[arg(long, env)]
    pub https_auto_cert_domain: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, env, default_value_t = Level::INFO)]
    pub log_level: Level,

    /// Maximum mail size in bytes, anything bigger will be ignored and not parsed
    #[arg(long, env, default_value_t = 1024 * 1024 * 1)]
    pub max_mail_size: u32,
}

impl Configuration {
    pub fn new() -> Self {
        Configuration::parse()
    }

    pub fn log(&self) {
        info!("Log Level: {}", self.log_level);

        info!("IMAP Host: {}", self.imap_host);
        info!("IMAP Port: {}", self.imap_port);
        info!("IMAP STARTTLS: {}", self.imap_starttls);
        info!("IMAP TLS CA Certificate File: {:?}", self.imap_tls_ca_certs);
        info!("IMAP TLS Disabled: {}", self.imap_disable_tls);
        info!("IMAP User: {}", self.imap_user);
        info!("IMAP Check Interval: {} seconds", self.imap_check_interval);
        info!("IMAP Body Request: {}", self.imap_body_request.to_string());
        info!("IMAP Chunk Size: {}", self.imap_chunk_size);
        info!("IMAP Timeout: {}", self.imap_timeout);

        info!("HTTP Binding: {}", self.http_server_binding);
        info!("HTTP Port: {}", self.http_server_port);
        info!("HTTP User: {}", self.http_server_user);

        info!("HTTPS Enabled: {}", self.https_auto_cert);
        info!("HTTPS Domain: {:?}", self.https_auto_cert_domain);
        info!("HTTPS Mail: {:?}", self.https_auto_cert_mail);
        info!("HTTPS Cache Dir: {:?}", self.https_auto_cert_cache);

        info!("Maximum Mail Body Size: {} bytes", self.max_mail_size);
    }
}

#[derive(Clone)]
pub enum ImapBodyRequest {
    /// RFC822 and BODY[]
    Default,
    /// RFC822
    Rfc822,
    /// BODY[]
    Body,
}

impl ImapBodyRequest {
    pub fn to_request_string(&self) -> String {
        match &self {
            ImapBodyRequest::Default => String::from("RFC822 BODY[]"),
            ImapBodyRequest::Rfc822 => String::from("RFC822"),
            ImapBodyRequest::Body => String::from("BODY[]"),
        }
    }
}

impl Display for ImapBodyRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match &self {
            ImapBodyRequest::Default => "default",
            ImapBodyRequest::Rfc822 => "rfc822",
            ImapBodyRequest::Body => "body",
        };
        write!(f, "{str}")
    }
}

impl FromStr for ImapBodyRequest {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let lower = value.to_lowercase();
        if lower == "default" {
            Ok(ImapBodyRequest::Default)
        } else if lower == "rfc822" {
            Ok(ImapBodyRequest::Rfc822)
        } else if lower == "body" {
            Ok(ImapBodyRequest::Body)
        } else {
            Err(format!("'{lower}' is not a valid value"))
        }
    }
}
