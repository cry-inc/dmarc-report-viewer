use anyhow::{Context, Result, ensure};
use clap::error::ErrorKind;
use clap::{CommandFactory, Parser, ValueEnum};
use cron::Schedule;
use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::{Level, info};

pub const HTTP_DEFAULT_PORT: u16 = 8080;
pub const HTTP_DEFAULT_BINDING: &str = "0.0.0.0";

const ADDITIONAL_ACCOUNTS_HELP: &str = "Additional IMAP accounts:
  More IMAP accounts can only be configured with numbered ENV variables, starting with number 2.
  Required per account: IMAP_USER_<N> and IMAP_PASSWORD_<N>.
  Optional, inherited from the first account when not set:
  IMAP_HOST_<N>, IMAP_PORT_<N>, IMAP_STARTTLS_<N>, IMAP_DISABLE_TLS_<N> and IMAP_TLS_CA_CERTS_<N>.
  Optional, not inherited: IMAP_FOLDER_<N> (default INBOX), IMAP_FOLDER_DMARC_<N> and IMAP_FOLDER_TLS_<N>.
  All other IMAP settings apply to all accounts.";

#[derive(Parser, Clone)]
#[command(version, about, long_about = None, after_long_help = ADDITIONAL_ACCOUNTS_HELP)]
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
    /// See IMAP folder depth setting when you also want to scan sub-folders for mails.
    #[arg(long, env, default_value = "INBOX")]
    pub imap_folder: String,

    /// Optional IMAP folder (will be only checked for DMARC reports).
    /// Will disable the normal default folder when set.
    /// See IMAP folder depth setting when you also want to scan sub-folders for mails.
    #[arg(long, env)]
    pub imap_folder_dmarc: Option<String>,

    /// Optional IMAP folder (will be only checked for SMTP TLS reports)
    /// Will disable the normal default folder when set.
    /// See IMAP folder depth setting when you also want to scan sub-folders for mails.
    #[arg(long, env)]
    pub imap_folder_tls: Option<String>,

    /// Maximum depth for recursive scanning of IMAP folders.
    /// The default value 0 disables recursive scanning and ignores all sub-folders.
    /// A depth of 1 will additionally scan the direct sub-folders of the configured folders,
    /// a depth of 2 will also include their sub-folders, etc.
    /// Applies to the single default IMAP folder as well as the dedicated DMARC and TLS folders.
    #[arg(long, env, default_value_t = 0)]
    pub imap_folder_depth: usize,

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
    #[arg(long, env, default_value_t = 1000)]
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

    /// All IMAP accounts, starting with the first one configured by the settings above.
    /// Additional accounts are read from numbered ENV variables like `IMAP_USER_2`.
    #[arg(skip)]
    pub imap_accounts: Vec<ImapAccount>,

    /// Embedded HTTP server port for web UI.
    /// Needs to be bigger than 0 because for 0 a random port will be used!
    #[arg(long, env, default_value_t = HTTP_DEFAULT_PORT)]
    pub http_server_port: u16,

    /// Embedded HTTP server binding for web UI.
    /// Needs to be a valid IPv4 or IPv6 address.
    /// The default will bind to all IPv4 IPs of the host.
    /// Use `[::]` to bind to all IPV6 IPs of the host.
    /// Use `127.0.0.1` (IPv4) or `[::1]` (IPv6) to make the server only available on localhost!
    #[arg(long, env, default_value = HTTP_DEFAULT_BINDING)]
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

    /// Disable duplicate report filtering.
    /// By default all DMARC and SMTP TLS reports are filtered for duplicates by report ID.
    /// This flag can be used to turn off the duplicate filter.
    #[arg(long, env)]
    pub disable_duplicate_filter: bool,

    /// Maximum mail size in bytes, anything bigger will be ignored and not parsed
    #[arg(long, env, default_value_t = 1000 * 1000 * 1)]
    pub max_mail_size: usize,

    /// Maximum uncompressed file size for mail attachments.
    /// The DMARC XML and SMTP TLS JSON report files are often compressed and their
    /// maximum uncompressed size needs to be limited to avoid compression bomb attacks.
    #[arg(long, env, default_value_t = 1000 * 1000 * 20)]
    pub max_uncompressed_size: usize,

    /// URL for optional web hook that is called via HTTP when a new mail is detected.
    /// Please note that this app does not have a persistent store for already known mails.
    /// When the application starts, all existing mails in the IMAP account are considered known.
    /// Only the subsequent updates that occur while the app is running will be able to detect new mails.
    /// The default HTTP method used is `POST`. You can change the method using another setting.
    /// The URL also supports template parameters that will be automatically replaced.
    /// Template parameters will be URL-encoded to avoid issues with any special characters.
    /// Please see the documentation of the optional hook body for a complete list of supported values.
    /// Example value: https://myserver.org:4443/api/my_endpoint?dmarc=[dmarc_reports]&sender=[sender]
    #[arg(long, env)]
    pub mail_web_hook_url: Option<String>,

    /// HTTP method (also known as HTTP verb) used for calling the web hook for new mails.
    /// Example values: POST, PUT, PATCH, etc.
    #[arg(long, env, default_value = "POST")]
    pub mail_web_hook_method: String,

    /// Optional custom HTTP headers used to for the outgoing web hook requests for new mails.
    /// You should specify them using a JSON object with the header name as key and the value for the content.
    /// Example value: `{"content-type": "application/json", "api-key": "my secret API key"}`
    #[arg(long, env)]
    pub mail_web_hook_headers: Option<String>,

    /// Optional custom HTTP body used to for the outgoing web hook requests for new mails.
    /// Should be an valid UTF8 or ASCII string.
    /// The body supports the following template parameters that will be replaced automatically:
    /// `[id]` ID of the mail used internally and by the web interface,
    /// `[uid]` Mail UID provided by IMAP server,
    /// `[sender]` Sender of the mail,
    /// `[subject]` Subject of the mail,
    /// `[folder]` IMAP folder of the mail,
    /// `[account]` IMAP account that received the mail,
    /// `[dmarc_reports]` Number of DMARC reports in the mail,
    /// `[tls_reports]` Number of SMTP TLS Reports in the mail
    #[arg(long, env)]
    pub mail_web_hook_body: Option<String>,

    /// DNS server address for resolving IPs to hostnames.
    /// Default is 1.1.1.1:53, which is the public Cloudflare DNS server.
    /// Do not forget to add the suffix with the port using a colon.
    #[arg(long, env, default_value = "1.1.1.1:53")]
    pub dns_server: SocketAddr,

    /// Timeout value for DNS queries in milliseconds.
    #[arg(long, env, default_value_t = 5000)]
    pub dns_timeout: u64,
}

impl Configuration {
    pub fn new() -> Self {
        let mut config = Configuration::parse();
        let first = ImapAccount {
            number: 1,
            host: config.imap_host.clone(),
            port: config.imap_port,
            user: config.imap_user.clone(),
            password: config.imap_password.clone(),
            starttls: config.imap_starttls,
            disable_tls: config.imap_disable_tls,
            tls_ca_certs: config.imap_tls_ca_certs.clone(),
            folder: config.imap_folder.clone(),
            folder_dmarc: config.imap_folder_dmarc.clone(),
            folder_tls: config.imap_folder_tls.clone(),
        };

        // Ignore ENV variables that are not valid Unicode instead of panicking
        let vars = std::env::vars_os()
            .filter_map(|(key, value)| Some((key.into_string().ok()?, value.into_string().ok()?)));
        match parse_additional_accounts(&first, vars) {
            Ok(additional) => {
                config.imap_accounts = vec![first];
                config.imap_accounts.extend(additional);
            }
            Err(err) => Configuration::command()
                .error(ErrorKind::ValueValidation, format!("{err:#}"))
                .exit(),
        }
        config
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
        info!("IMAP Folder Depth: {}", self.imap_folder_depth);
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
        for account in self.imap_accounts.iter().skip(1) {
            let number = account.number;
            info!("IMAP Account {number} Host: {}", account.host);
            info!("IMAP Account {number} Port: {}", account.port);
            info!("IMAP Account {number} STARTTLS: {}", account.starttls);
            info!(
                "IMAP Account {number} TLS CA Certificate File: {:?}",
                account.tls_ca_certs
            );
            info!(
                "IMAP Account {number} TLS Disabled: {}",
                account.disable_tls
            );
            info!("IMAP Account {number} User: {}", account.user);
            info!("IMAP Account {number} Folder: {}", account.folder);
            info!(
                "IMAP Account {number} DMARC Folder: {:?}",
                account.folder_dmarc
            );
            info!("IMAP Account {number} TLS Folder: {:?}", account.folder_tls);
        }

        info!("HTTP Binding: {}", self.http_server_binding);
        info!("HTTP Port: {}", self.http_server_port);
        info!("HTTP User: {}", self.http_server_user);

        info!("HTTPS Enabled: {}", self.https_auto_cert);
        info!("HTTPS Domain: {:?}", self.https_auto_cert_domain);
        info!("HTTPS Mail: {:?}", self.https_auto_cert_mail);
        info!("HTTPS Cache Dir: {:?}", self.https_auto_cert_cache);

        info!(
            "Disable Duplicate Filter: {}",
            self.disable_duplicate_filter
        );

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

/// Connection and folder settings of a single IMAP account
#[derive(Clone)]
pub struct ImapAccount {
    /// Number of the account, the first account has the number 1
    pub number: usize,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub starttls: bool,
    pub disable_tls: bool,
    pub tls_ca_certs: Option<PathBuf>,
    pub folder: String,
    pub folder_dmarc: Option<String>,
    pub folder_tls: Option<String>,
}

/// Reads additional IMAP accounts from numbered ENV variables like `IMAP_USER_2`.
/// Connection settings that are not set are inherited from the first account.
/// User and password are always required, so the password of the first account
/// is never sent to a different server or user by accident.
fn parse_additional_accounts(
    first: &ImapAccount,
    vars: impl Iterator<Item = (String, String)>,
) -> Result<Vec<ImapAccount>> {
    // Group the settings by account number
    let mut accounts: BTreeMap<usize, HashMap<String, String>> = BTreeMap::new();
    for (key, value) in vars {
        let Some((name, number)) = key
            .strip_prefix("IMAP_")
            .and_then(|rest| rest.rsplit_once('_'))
        else {
            continue;
        };
        let Ok(number) = number.parse::<usize>() else {
            continue;
        };
        ensure!(
            number >= 2,
            "Invalid ENV variable {key}: Additional IMAP accounts start with number 2"
        );
        accounts
            .entry(number)
            .or_default()
            .insert(name.to_string(), value);
    }

    let mut result = Vec::new();
    for (number, mut settings) in accounts {
        let parse_bool = |name: &str, value: Option<String>, default: bool| match value {
            Some(value) => value.parse::<bool>().context(format!(
                "Invalid value for IMAP_{name}_{number}, expected true or false"
            )),
            None => Ok(default),
        };
        let account = ImapAccount {
            number,
            host: settings.remove("HOST").unwrap_or(first.host.clone()),
            port: match settings.remove("PORT") {
                Some(port) => port
                    .parse()
                    .context(format!("Invalid value for IMAP_PORT_{number}"))?,
                None => first.port,
            },
            user: settings
                .remove("USER")
                .context(format!("Missing ENV variable IMAP_USER_{number}"))?,
            password: settings
                .remove("PASSWORD")
                .context(format!("Missing ENV variable IMAP_PASSWORD_{number}"))?,
            starttls: parse_bool("STARTTLS", settings.remove("STARTTLS"), first.starttls)?,
            disable_tls: parse_bool(
                "DISABLE_TLS",
                settings.remove("DISABLE_TLS"),
                first.disable_tls,
            )?,
            tls_ca_certs: settings
                .remove("TLS_CA_CERTS")
                .map(PathBuf::from)
                .or(first.tls_ca_certs.clone()),
            folder: settings.remove("FOLDER").unwrap_or(String::from("INBOX")),
            folder_dmarc: settings.remove("FOLDER_DMARC"),
            folder_tls: settings.remove("FOLDER_TLS"),
        };
        ensure!(
            settings.is_empty(),
            "Unsupported ENV variable(s) for IMAP account {number}: {:?}",
            settings.keys().collect::<Vec<_>>()
        );
        ensure!(
            !(account.starttls && account.disable_tls),
            "IMAP account {number} cannot use STARTTLS and disabled TLS at the same time"
        );
        result.push(account);
    }
    Ok(result)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn first_account() -> ImapAccount {
        ImapAccount {
            number: 1,
            host: String::from("imap.first.org"),
            port: 143,
            user: String::from("dmarc"),
            password: String::from("secret"),
            starttls: true,
            disable_tls: false,
            tls_ca_certs: None,
            folder: String::from("reports"),
            folder_dmarc: Some(String::from("dmarc")),
            folder_tls: None,
        }
    }

    fn parse(vars: &[(&str, &str)]) -> Result<Vec<ImapAccount>> {
        let vars = vars
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()));
        parse_additional_accounts(&first_account(), vars)
    }

    #[test]
    fn additional_accounts() {
        // Settings without number are ignored
        let none = parse(&[("IMAP_CHUNK_SIZE", "5"), ("IMAP_USER", "a")]).unwrap();
        assert!(none.is_empty());

        let accounts = parse(&[
            ("IMAP_USER_3", "tls"),
            ("IMAP_PASSWORD_3", "pw3"),
            ("IMAP_HOST_3", "imap.third.org"),
            ("IMAP_PORT_3", "993"),
            ("IMAP_STARTTLS_3", "false"),
            ("IMAP_FOLDER_TLS_3", "tls"),
            ("IMAP_USER_2", "other"),
            ("IMAP_PASSWORD_2", "pw2"),
        ])
        .unwrap();
        assert_eq!(accounts.len(), 2);

        // Connection settings are inherited, folders are not
        let second = &accounts[0];
        assert_eq!(second.number, 2);
        assert_eq!(second.host, "imap.first.org");
        assert_eq!(second.port, 143);
        assert_eq!(second.user, "other");
        assert_eq!(second.password, "pw2");
        assert!(second.starttls);
        assert_eq!(second.folder, "INBOX");
        assert_eq!(second.folder_dmarc, None);

        let third = &accounts[1];
        assert_eq!(third.number, 3);
        assert_eq!(third.host, "imap.third.org");
        assert_eq!(third.port, 993);
        assert!(!third.starttls);
        assert_eq!(third.folder_tls.as_deref(), Some("tls"));
    }

    #[test]
    fn invalid_additional_accounts() {
        // User and password are never inherited
        assert!(parse(&[("IMAP_USER_2", "other")]).is_err());
        assert!(parse(&[("IMAP_HOST_2", "h"), ("IMAP_PASSWORD_2", "pw")]).is_err());

        let with = |extra| parse(&[("IMAP_USER_2", "other"), ("IMAP_PASSWORD_2", "pw"), extra]);
        assert!(with(("IMAP_FOLDER_2", "reports")).is_ok());
        assert!(with(("IMAP_PORT_2", "abc")).is_err());
        assert!(with(("IMAP_HOSTNAME_2", "typo")).is_err());
        assert!(with(("IMAP_USER_1", "first")).is_err());
        // STARTTLS is inherited from the first account
        assert!(with(("IMAP_DISABLE_TLS_2", "true")).is_err());
    }
}
