use clap::{Parser, ValueEnum};
use cron::Schedule;
use std::path::PathBuf;
use tracing::{Level, info};

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

    /// IMAP folder, will be used to look for all kinds of reports (DMARC and SMTP TLS).
    /// Will be only used if the dedicated folders for TLS and DMARC are not set!
    #[arg(long, env, default_value = "INBOX")]
    pub imap_folder: String,

    /// Optional IMAP folder (will be only checked for DMARC reports).
    /// Will disable the normal default folder when set.
    #[arg(long, env)]
    pub imap_folder_dmarc: Option<String>,

    /// Optional IMAP folder (will be only checked for SMTP TLS reports)
    /// Will disable the normal default folder when set.
    #[arg(long, env)]
    pub imap_folder_tls: Option<String>,

    /// Method of requesting the mail body from the IMAP server.
    /// The default should work for most IMAP servers.
    /// Only try other values if you have issues with missing mail bodies.
    #[arg(long, env, default_value = "default")]
    pub imap_body_request: ImapBodyRequest,

    /// TCP connection timeout for IMAP server in seconds
    #[arg(long, env, default_value_t = 10)]
    pub imap_timeout: u64,

    /// Number of mails downloaded in one chunk, must be bigger than 0.
    /// The default value should work for most IMAP servers.
    /// Try lower values in case of warnings like "Unable to fetch some mails from chunk"!
    #[arg(long, env, default_value_t = 2000)]
    pub imap_chunk_size: usize,

    /// Interval between checking for new reports in IMAP inbox in seconds
    #[arg(long, env, default_value_t = 1800)]
    pub imap_check_interval: u64,

    /// Schedule for checking the IMAP inbox.
    /// Specified as cron expression string (in Local time).
    /// Will replace and override the IMAP check interval if specified.
    /// Columns: sec, min, hour, day of month, month, day of week, year.
    /// When running the official Docker image the local time zone will be UTC.
    /// To change this, set the `TZ` ENV var. Since the image comes without
    /// time zone data, you also need to mount the host folder
    /// `/usr/share/zoneinfo` into the container.
    #[arg(long, env)]
    pub imap_check_schedule: Option<Schedule>,

    /// Embedded HTTP server port for web UI.
    /// Needs to be bigger than 0 because for 0 a random port will be used!
    #[arg(long, env, default_value_t = 8080)]
    pub http_server_port: u16,

    /// Embedded HTTP server binding for web UI.
    /// Needs to be a valid IPv4 or IPv6 address.
    /// The default will bind to all IPv4 IPs of the host.
    /// Use `[::]` to bind to all IPV6 IPs of the host.
    /// Use `127.0.0.1` (IPv4) or `[::1]` (IPv6) to make the server only available on localhost!
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

    /// URL for optional web hook that is called via HTTP when a new mail is detected.
    /// Please note that this app does not have a persistent store for already known mails.
    /// When the application starts, all existing mails in the IMAP account are considered known.
    /// Only the subsequent updates that occur while the app is running will be able to detect new mails.
    /// The URL only supports plain HTTP (no HTTPS) and will reveive a HTTP request.
    /// The default HTTP method used is `POST`. You can change the method using another setting.
    /// The URL also supports template parameters that will be automatically replaced.
    /// They will be always URL-encoded to avoid issues with special characters.
    /// Please see the documentation of the optional hook body for a complete list of supported values.
    /// Example value: http://myserver.org/api/my_endpoint?dmarc=[dmarc_reports]&sender=[sender]
    #[arg(long, env)]
    pub mail_web_hook_url: Option<String>,

    /// HTTP method (also known as HTTP verb) used for calling the web hook for new mails.
    /// Example values: POST, PUT, PATCH, etc.
    #[arg(long, env, default_value = "POST")]
    pub mail_web_hook_method: String,

    /// Optional custom HTTP headers used to for the outgoing web hook requests for new mails.
    /// You should specify them using a JSON object with the header name as key and the value for the content.
    /// Example value: {"content-type": "application/json", "api-key": "my secret API key"}
    #[arg(long, env)]
    pub mail_web_hook_headers: Option<String>,

    /// Optional custom HTTP body used to for the outgoing web hook requests for new mails.
    /// Should be an valid UTF8 or ASCII string.
    /// The body supports the following template parameters that will be replaced automatically:
    /// - `[id]` ID of the mail used internally and by the web interface
    /// - `[uid]` Mail UID provided by IMAP server
    /// - `[sender]` Sender of the mail
    /// - `[subject]` Subject of the mail
    /// - `[folder]` IMAP folder of the mail
    /// - `[account]` IMAP account that received the mail
    /// - `[dmarc_reports]` Number of DMARC reports in the mail
    /// - `[tls_reports]` Number of SMTP TLS Reports in the mail
    #[arg(long, env)]
    pub mail_web_hook_body: Option<String>,
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
        info!("IMAP Folder: {}", self.imap_folder);
        info!("IMAP DMARC Folder: {:?}", self.imap_folder_dmarc);
        info!("IMAP TLS Folder: {:?}", self.imap_folder_tls);
        info!("IMAP Check Interval: {} seconds", self.imap_check_interval);
        info!(
            "IMAP Schedule: {}",
            self.imap_check_schedule
                .as_ref()
                .map(|s| s.source().to_string())
                .unwrap_or(String::from("None"))
        );
        info!("IMAP Body Request: {:?}", self.imap_body_request);
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

        info!("Mail Web Hook URL: {:?}", self.mail_web_hook_url);
        info!("Mail Web Hook Method: {}", self.mail_web_hook_method);
        info!(
            "Mail Web Hook Headers: {}",
            if self.mail_web_hook_headers.is_some() {
                "Hidden"
            } else {
                "None"
            }
        );
        info!(
            "Mail Web Hook Body: {}",
            if self.mail_web_hook_body.is_some() {
                "Hidden"
            } else {
                "None"
            }
        );
    }
}

#[derive(Clone, ValueEnum, Debug, Default)]
pub enum ImapBodyRequest {
    /// RFC822 and BODY[]
    #[default]
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
