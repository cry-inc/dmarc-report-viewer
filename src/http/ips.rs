use crate::geolocate::Location;
use crate::state::AppState;
use crate::whois::WhoIsIp;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn dns_single_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(ip): Path<IpAddr>,
) -> impl IntoResponse {
    // First get DNS client from state and then send a new query...
    let dns_client = {
        let locked = state.lock().await;
        locked.dns_client.clone()
    };
    let result = dns_client.host_from_ip(ip).await;

    // Check for any DNS request errors
    let Ok(response) = result else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain")],
            String::from("DNS lookup failed"),
        );
    };

    if let Some(host_name) = response {
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

pub async fn dns_batch_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(ips): Json<Vec<IpAddr>>,
) -> impl IntoResponse {
    // Check number of IPs
    const MAX_IP_COUNT: usize = 100;
    if ips.len() > MAX_IP_COUNT {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "text/plain")],
            format!("Requests can only contain up to {MAX_IP_COUNT} IPs"),
        );
    }

    // Get DNS client from state
    let dns_client = {
        let locked = state.lock().await;
        locked.dns_client.clone()
    };

    // Spawn tasks for all requests
    let mut handles = Vec::with_capacity(ips.len());
    for ip in ips {
        let dns_client = dns_client.clone();
        let handle = tokio::spawn(async move { dns_client.host_from_ip(ip).await });
        handles.push(handle);
    }

    // Join the tasks with the results again
    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        if let Ok(result) = handle.await {
            // Errors will be also mapped to None
            let flat_result = result.ok().flatten();
            results.push(flat_result);
        } else {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain")],
                String::from("Failed to join DNS query task"),
            );
        }
    }

    // Serialize results to JSON
    if let Ok(json) = serde_json::to_string_pretty(&results) {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            json,
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain")],
            String::from("Unable to serialize result"),
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
