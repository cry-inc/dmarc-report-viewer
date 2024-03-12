use clap::Parser;

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
/// DMARC Report Analyzer
pub struct Configuration {
    /// Host name or domain of the IMAP server with the DMARC reports inbox
    #[arg(short = 's', long, env)]
    pub imap_host: String,

    /// User name of the IMAP inbox with the DMARC reports
    #[arg(short = 'u', long, env)]
    pub imap_user: String,

    /// Password of the IMAP inbox with the DMARC reports
    #[arg(short = 'p', long, env)]
    pub imap_password: String,

    /// TLS encrypted port of the IMAP server
    #[arg(short = 't', long, env, default_value = "993")]
    pub imap_port: u16,

    /// Interval between checking for new reports in IMAP inbox in seconds
    #[arg(short = 'i', long, env, default_value = "300")]
    pub imap_check_interval: u64,

    /// Embedded HTTP server port for web UI
    #[arg(short = 'w', long, env, default_value = "8080")]
    pub http_server_port: u16,

    /// Embedded HTTP server binding for web UI
    #[arg(short = 'b', long, env, default_value = "0.0.0.0")]
    pub http_server_binding: String,

    /// Username for the HTTP server basic auth login
    #[arg(short = 'l', long, env, default_value = "dmarc")]
    pub http_server_user: String,

    /// Password for the HTTP server basic auth login.
    /// Use empty string to disable (not recommended).
    #[arg(short = 'a', long, env)]
    pub http_server_password: String,
}

impl Configuration {
    pub fn new() -> Self {
        Configuration::parse()
    }
}
