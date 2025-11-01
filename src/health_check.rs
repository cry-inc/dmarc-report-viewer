use crate::http_client::http_request;
use clap::Parser;
use hyper::{Method, StatusCode};
use std::collections::HashMap;

#[derive(Parser)]
#[command(ignore_errors = true)]
struct HealthCheckArgs {
    #[arg(long)]
    pub health_check: bool,
}

pub async fn run_health_check_if_requested() {
    let args = HealthCheckArgs::parse();
    if args.health_check {
        run_health_check().await;
    }
}

async fn run_health_check() {
    let url = "http://127.0.0.1:8080/health";
    let headers = HashMap::new();
    let body = Vec::new();
    let result = http_request(Method::GET, url, &headers, body).await;
    match result {
        Ok((status, ..)) => {
            if status == StatusCode::OK {
                println!("Health via {url} check successful!");
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
