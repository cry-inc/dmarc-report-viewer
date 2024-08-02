use clap::Parser;
use tracing::Level;

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

    /// TCP connection timeout for IMAP server in seconds
    #[arg(long, env, default_value_t = 10)]
    pub imap_timeout: u64,

    /// Interval between checking for new reports in IMAP inbox in seconds
    #[arg(long, env, default_value_t = 1000)]
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
    pub https_auto_cert_cache: Option<String>,

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
}
