use crate::geolocate::Location;
use crate::state::AppState;
use crate::whois::WhoIsIp;
use axum::extract::Path;
use axum::extract::State;
use axum::http::header;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use dns_lookup::lookup_addr;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;

pub async fn to_dns_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(ip): Path<IpAddr>,
) -> impl IntoResponse {
    // Check cache
    let cached = {
        let app = state.lock().await;
        app.ip_dns_cache.get(&ip).map(|dns| dns.to_string())
    };

    let result = if let Some(host_name) = cached {
        // Found result in cache!
        Some(host_name)
    } else {
        // Nothing in cache, send new DNS request!
        // Do not block here and use a special async task
        // where blocking calls are acceptable
        let handle = spawn_blocking(move || lookup_addr(&ip));

        // Join async task
        let Ok(result) = handle.await else {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain")],
                String::from("DNS lookup failed"),
            );
        };

        // Cache any positive result
        if let Ok(host_name) = result {
            let mut app = state.lock().await;
            app.ip_dns_cache.insert(ip, host_name.clone());
            Some(host_name)
        } else {
            None
        }
    };

    if let Some(host_name) = result {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain")],
            host_name,
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            String::from("n/a"),
        )
    }
}

pub async fn to_location_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(ip): Path<IpAddr>,
) -> impl IntoResponse {
    // Check cache
    let cached = {
        let app = state.lock().await;
        app.ip_location_cache.get(&ip).cloned()
    };

    let result = if let Some(location) = cached {
        // Found result in cache!
        Some(location)
    } else {
        // Nothing in cache, send new request!
        let Ok(result) = Location::from_ip(&ip).await else {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain")],
                String::from("Failed to locate IP"),
            );
        };

        // Cache any positive result
        if let Some(location) = result {
            let mut app = state.lock().await;
            app.ip_location_cache.insert(ip, location.clone());
            Some(location)
        } else {
            None
        }
    };

    let Some(location) = result else {
        return (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            String::from("No info found"),
        );
    };

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_string_pretty(&location).expect("Failed to serialize JSON"),
    )
}

pub async fn to_whois_handler(Path(ip): Path<IpAddr>) -> impl IntoResponse {
    let whois = WhoIsIp::default();
    let Ok(whois) = whois.lookup(&ip).await else {
        return (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            String::from("Failed to look up IP"),
        );
    };

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain")],
        whois,
    )
}
