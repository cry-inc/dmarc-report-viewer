use crate::config::{HTTP_DEFAULT_BINDING, HTTP_DEFAULT_PORT};
use crate::http_client::http_request;
use clap::Parser;
use hyper::{Method, StatusCode};
use std::collections::HashMap;

#[derive(Parser)]
#[command(ignore_errors = true, disable_help_flag = true)]
struct HealthCheckArgs {
    /// Set to enable health check
    #[arg(long)]
    pub health_check: bool,

    /// See `Configuration::http_server_port`
    #[arg(long, env, default_value_t = HTTP_DEFAULT_PORT)]
    pub http_server_port: u16,

    /// See `Configuration::http_server_binding`
    #[arg(long, env, default_value = HTTP_DEFAULT_BINDING)]
    pub http_server_binding: String,

    /// See `Configuration::https_auto_cert`
    #[arg(long, env, requires = "https_auto_cert_domain")]
    pub https_auto_cert: bool,

    /// See `Configuration::https_auto_cert_domain`
    #[arg(long, env)]
    pub https_auto_cert_domain: Option<String>,
}

pub async fn run_health_check_if_requested() {
    let args = HealthCheckArgs::parse();
    if args.health_check {
        run_health_check(&args).await;
    }
}

fn create_check_url(args: &HealthCheckArgs) -> String {
    let mut port = args.http_server_port;
    let mut protocol = String::from("http");
    let mut host = match args.http_server_binding.as_str() {
        "127.0.0.1" => String::from("127.0.0.1"),
        "0.0.0.0" => String::from("127.0.0.1"),
        "[::1]" => String::from("[::1]"),
        "[::]" => String::from("[::1]"),
        other => String::from(other),
    };
    if args.https_auto_cert
        && let Some(https_host) = &args.https_auto_cert_domain
    {
        // When the HTTPS feature with automatic certificates is enabled,
        // we need to use the HTTPS protocol to check via public host name.
        // Otherwise the HTTPS request will fail because the host does not match.
        // Since we use the public host, we also need to use the public port,
        // which is always 443 (this is required by the certificate challenge).
        protocol = String::from("https");
        port = 443;
        host = https_host.to_string();
    }
    format!("{protocol}://{host}:{port}/health")
}

async fn run_health_check(args: &HealthCheckArgs) {
    let url = create_check_url(args);
    println!("Checking health via {url}...");
    let headers = HashMap::new();
    let body = Vec::new();
    let result = http_request(Method::GET, &url, &headers, body).await;
    match result {
        Ok((status, ..)) => {
            if status == StatusCode::OK {
                println!("Health check successful!");
                std::process::exit(0);
            } else {
                eprintln!("Health check returned unexpected status code: {status}");
                std::process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("Health check request failed: {err:#}");
            std::process::exit(1);
        }
    }
}
