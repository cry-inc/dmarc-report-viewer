use crate::config::Configuration;
use crate::http_client::http_request;
use anyhow::{Context, Result, anyhow, bail, ensure};
use axum::Router;
use axum::extract::{Query, Request, State};
use axum::http::header::{ACCEPT, COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::middleware::Next;
use axum::response::{AppendHeaders, IntoResponse, Redirect, Response};
use axum::routing::get;
use base64::prelude::*;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tokio::time::timeout;
use tokio_rustls::rustls::crypto::aws_lc_rs;
use tracing::{info, warn};

const SESSION_COOKIE: &str = "dmarc_session";
const STATE_COOKIE: &str = "dmarc_oidc_state";
const CALLBACK_PATH: &str = "oidc/callback";
const SESSION_LIFETIME: Duration = Duration::from_secs(8 * 60 * 60);
const LOGIN_LIFETIME: Duration = Duration::from_secs(10 * 60);
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

/// Upper limit for unfinished logins, they can be started without authentication
const MAX_PENDING_LOGINS: usize = 1000;

/// Subset of the OpenID Connect discovery document
#[derive(Deserialize)]
struct Discovery {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    id_token: String,
}

#[derive(Deserialize)]
struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Audience {
    One(String),
    Many(Vec<String>),
}

#[derive(Deserialize)]
struct Claims {
    iss: String,
    aud: Audience,
    exp: f64,
    nonce: Option<String>,
    azp: Option<String>,
}

/// Login that was sent to the provider and is waiting for the callback
struct PendingLogin {
    nonce: String,
    code_verifier: String,
    expires: Instant,
}

/// OpenID Connect login using the authorization code flow with PKCE.
/// Logins and sessions are only kept in memory and are lost on restart.
pub struct Oidc {
    issuer: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    authorization_endpoint: String,
    token_endpoint: String,

    /// Public path of the application root with trailing slash
    base_path: String,

    secure_cookies: bool,

    /// Pending logins keyed by state parameter
    pending_logins: Mutex<HashMap<String, PendingLogin>>,

    /// Expiration times of the sessions keyed by session ID
    sessions: Mutex<HashMap<String, Instant>>,
}

impl Oidc {
    /// Returns `None` if OIDC is not configured.
    /// Fetches the discovery document of the provider otherwise.
    pub async fn discover(config: &Configuration) -> Result<Option<Arc<Self>>> {
        let Some(issuer) = config.oidc_issuer_url.as_deref() else {
            return Ok(None);
        };
        ensure!(
            config.http_server_password.is_empty(),
            "OIDC and basic auth cannot be combined, remove the HTTP server password or the OIDC settings"
        );
        let client_id = config
            .oidc_client_id
            .clone()
            .context("OIDC client ID is missing in configuration")?;
        let client_secret = config
            .oidc_client_secret
            .clone()
            .context("OIDC client secret is missing in configuration")?;
        let redirect_uri = config
            .oidc_redirect_uri
            .clone()
            .context("OIDC redirect URI is missing in configuration")?;
        let (base_path, secure_cookies) = parse_redirect_uri(&redirect_uri)?;

        let issuer = issuer.trim_end_matches('/');
        let url = format!("{issuer}/.well-known/openid-configuration");
        let headers = HashMap::new();
        let request = http_request(Method::GET, &url, &headers, Vec::new());
        let (status, _, body) = timeout(HTTP_TIMEOUT, request)
            .await
            .context("OIDC discovery request timed out")?
            .context("OIDC discovery request failed")?;
        ensure!(
            status == StatusCode::OK,
            "OIDC discovery returned HTTP status {status}"
        );
        let discovery: Discovery =
            serde_json::from_slice(&body).context("Failed to parse OIDC discovery document")?;
        ensure!(
            discovery.issuer.trim_end_matches('/') == issuer,
            "OIDC discovery document is for a different issuer"
        );
        if !discovery.token_endpoint.starts_with("https://") {
            // The ID token signature is not checked, its origin is only proven by TLS
            warn!(
                "OIDC token endpoint does not use HTTPS: This is insecure, use for testing only!"
            );
        }
        info!(
            "OIDC authentication enabled for issuer {}",
            discovery.issuer
        );

        Ok(Some(Arc::new(Self {
            issuer: discovery.issuer,
            client_id,
            client_secret,
            redirect_uri,
            authorization_endpoint: discovery.authorization_endpoint,
            token_endpoint: discovery.token_endpoint,
            base_path,
            secure_cookies,
            pending_logins: Mutex::new(HashMap::new()),
            sessions: Mutex::new(HashMap::new()),
        })))
    }

    /// Unprotected routes required for the login flow
    pub fn router(self: &Arc<Self>) -> Router {
        Router::new()
            .route("/oidc/login", get(login))
            .route("/oidc/callback", get(callback))
            .route("/oidc/logout", get(logout))
            .with_state(self.clone())
    }

    fn has_session(&self, headers: &HeaderMap) -> bool {
        let Some(id) = get_cookie(headers, SESSION_COOKIE) else {
            return false;
        };
        self.sessions
            .lock()
            .expect("Failed to lock sessions")
            .get(id)
            .is_some_and(|expires| *expires > Instant::now())
    }

    fn create_cookie(&self, name: &str, value: &str, max_age: Duration) -> String {
        format!(
            "{name}={value}; Path={}; Max-Age={}; HttpOnly; SameSite=Lax{}",
            self.base_path,
            max_age.as_secs(),
            if self.secure_cookies { "; Secure" } else { "" }
        )
    }

    /// Returns the URL of the provider login page and the state parameter
    fn start_login(&self) -> Result<(String, String)> {
        let state = random_token()?;
        let nonce = random_token()?;
        let code_verifier = random_token()?;
        let code_challenge = BASE64_URL_SAFE_NO_PAD.encode(Sha256::digest(&code_verifier));
        let separator = if self.authorization_endpoint.contains('?') {
            '&'
        } else {
            '?'
        };
        let url = format!(
            "{}{separator}response_type=code&scope=openid&client_id={}&redirect_uri={}\
            &state={state}&nonce={nonce}&code_challenge={code_challenge}&code_challenge_method=S256",
            self.authorization_endpoint,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
        );

        let now = Instant::now();
        let mut pending_logins = self
            .pending_logins
            .lock()
            .expect("Failed to lock pending logins");
        pending_logins.retain(|_, login| login.expires > now);
        ensure!(
            pending_logins.len() < MAX_PENDING_LOGINS,
            "Too many pending logins"
        );
        pending_logins.insert(
            state.clone(),
            PendingLogin {
                nonce,
                code_verifier,
                expires: now + LOGIN_LIFETIME,
            },
        );
        Ok((url, state))
    }

    /// Returns the ID of the new session
    async fn finish_login(&self, headers: &HeaderMap, params: CallbackParams) -> Result<String> {
        if let Some(error) = params.error {
            bail!("Provider returned error {error:?}");
        }
        let state = params.state.context("State parameter is missing")?;
        let code = params.code.context("Code parameter is missing")?;

        // The state cookie binds the login to the browser that started it
        ensure!(
            get_cookie(headers, STATE_COOKIE) == Some(state.as_str()),
            "State does not match the login started by this browser"
        );
        let login = self
            .pending_logins
            .lock()
            .expect("Failed to lock pending logins")
            .remove(&state)
            .filter(|login| login.expires > Instant::now())
            .context("Login is unknown or expired")?;

        let claims = self.request_claims(&code, &login.code_verifier).await?;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("Failed to get Unix time stamp")?
            .as_secs();
        validate_claims(&claims, &self.issuer, &self.client_id, &login.nonce, now)?;

        let id = random_token()?;
        let now = Instant::now();
        let mut sessions = self.sessions.lock().expect("Failed to lock sessions");
        sessions.retain(|_, expires| *expires > now);
        sessions.insert(id.clone(), now + SESSION_LIFETIME);
        Ok(id)
    }

    /// Exchanges the authorization code for the claims of the ID token
    async fn request_claims(&self, code: &str, code_verifier: &str) -> Result<Claims> {
        let body = format!(
            "grant_type=authorization_code&code={}&redirect_uri={}&code_verifier={code_verifier}",
            urlencoding::encode(code),
            urlencoding::encode(&self.redirect_uri),
        );
        let credentials = BASE64_STANDARD.encode(format!(
            "{}:{}",
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.client_secret)
        ));
        let headers = HashMap::from([
            (
                String::from("content-type"),
                String::from("application/x-www-form-urlencoded"),
            ),
            (String::from("accept"), String::from("application/json")),
            (
                String::from("authorization"),
                format!("Basic {credentials}"),
            ),
        ]);
        let request = http_request(
            Method::POST,
            &self.token_endpoint,
            &headers,
            body.into_bytes(),
        );
        let (status, _, body) = timeout(HTTP_TIMEOUT, request)
            .await
            .context("Token request timed out")?
            .context("Token request failed")?;
        ensure!(
            status == StatusCode::OK,
            "Token endpoint returned HTTP status {status}"
        );
        let response: TokenResponse =
            serde_json::from_slice(&body).context("Failed to parse token response")?;
        decode_claims(&response.id_token)
    }
}

/// Middleware to protect routes with an OIDC session.
/// Browser page loads are redirected to the login, everything else gets a 401.
pub async fn auth_middleware(
    State(oidc): State<Arc<Oidc>>,
    request: Request,
    next: Next,
) -> Response {
    if oidc.has_session(request.headers()) {
        return next.run(request).await;
    }
    let wants_html = request
        .headers()
        .get(ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("text/html"));
    if request.method() == Method::GET && wants_html {
        Redirect::to(&format!("{}oidc/login", oidc.base_path)).into_response()
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

async fn login(State(oidc): State<Arc<Oidc>>) -> Response {
    match oidc.start_login() {
        Ok((url, state)) => {
            let cookie = oidc.create_cookie(STATE_COOKIE, &state, LOGIN_LIFETIME);
            ([(SET_COOKIE, cookie)], Redirect::to(&url)).into_response()
        }
        Err(err) => {
            warn!("Failed to start OIDC login: {err:#}");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

async fn callback(
    State(oidc): State<Arc<Oidc>>,
    headers: HeaderMap,
    Query(params): Query<CallbackParams>,
) -> Response {
    match oidc.finish_login(&headers, params).await {
        Ok(session) => {
            let cookies = AppendHeaders([
                (
                    SET_COOKIE,
                    oidc.create_cookie(SESSION_COOKIE, &session, SESSION_LIFETIME),
                ),
                (
                    SET_COOKIE,
                    oidc.create_cookie(STATE_COOKIE, "", Duration::ZERO),
                ),
            ]);
            (cookies, Redirect::to(&oidc.base_path)).into_response()
        }
        Err(err) => {
            warn!("OIDC login failed: {err:#}");
            (StatusCode::UNAUTHORIZED, "Login failed").into_response()
        }
    }
}

async fn logout(State(oidc): State<Arc<Oidc>>, headers: HeaderMap) -> Response {
    if let Some(id) = get_cookie(&headers, SESSION_COOKIE) {
        oidc.sessions
            .lock()
            .expect("Failed to lock sessions")
            .remove(id);
    }
    let cookie = oidc.create_cookie(SESSION_COOKIE, "", Duration::ZERO);
    ([(SET_COOKIE, cookie)], "Logged out").into_response()
}

/// Returns the public base path of the application and if HTTPS is used
fn parse_redirect_uri(redirect_uri: &str) -> Result<(String, bool)> {
    let uri = redirect_uri
        .parse::<Uri>()
        .context("Failed to parse OIDC redirect URI")?;
    let base_path = uri
        .path()
        .strip_suffix(CALLBACK_PATH)
        .filter(|path| path.ends_with('/'))
        .context("OIDC redirect URI must end with /oidc/callback")?;
    Ok((base_path.to_string(), uri.scheme_str() == Some("https")))
}

fn get_cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|cookie| cookie.trim().split_once('='))
        .find(|(cookie_name, _)| *cookie_name == name)
        .map(|(_, value)| value)
}

/// Creates an URL-safe token from 256 bits of secure randomness
fn random_token() -> Result<String> {
    let mut bytes = [0; 32];
    aws_lc_rs::default_provider()
        .secure_random
        .fill(&mut bytes)
        .map_err(|_| anyhow!("Failed to get secure random bytes"))?;
    Ok(BASE64_URL_SAFE_NO_PAD.encode(bytes))
}

/// Extracts the claims of an ID token without checking the signature.
/// OpenID Connect Core 3.1.3.7 allows to rely on the TLS server validation
/// instead, because the token is received directly from the token endpoint.
fn decode_claims(id_token: &str) -> Result<Claims> {
    let payload = id_token
        .split('.')
        .nth(1)
        .context("ID token is not a JWT")?;
    let json = BASE64_URL_SAFE_NO_PAD
        .decode(payload)
        .context("Failed to decode ID token")?;
    serde_json::from_slice(&json).context("Failed to parse ID token claims")
}

fn validate_claims(
    claims: &Claims,
    issuer: &str,
    client_id: &str,
    nonce: &str,
    now: u64,
) -> Result<()> {
    ensure!(claims.iss == issuer, "ID token has unexpected issuer");
    let audience_matches = match &claims.aud {
        Audience::One(audience) => audience == client_id,
        Audience::Many(audiences) => audiences.iter().any(|audience| audience == client_id),
    };
    ensure!(audience_matches, "ID token has unexpected audience");
    ensure!(
        claims.azp.as_deref().is_none_or(|azp| azp == client_id),
        "ID token has unexpected authorized party"
    );
    ensure!(claims.exp > now as f64, "ID token is expired");
    ensure!(
        claims.nonce.as_deref() == Some(nonce),
        "ID token has unexpected nonce"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_oidc() -> Oidc {
        Oidc {
            issuer: String::from("https://idp.example.org/realms/test"),
            client_id: String::from("my client"),
            client_secret: String::from("secret"),
            redirect_uri: String::from("https://dmarc.example.org/oidc/callback"),
            authorization_endpoint: String::from("https://idp.example.org/auth"),
            token_endpoint: String::from("https://idp.example.org/token"),
            base_path: String::from("/"),
            secure_cookies: true,
            pending_logins: Mutex::new(HashMap::new()),
            sessions: Mutex::new(HashMap::new()),
        }
    }

    fn create_claims() -> serde_json::Value {
        json!({
            "iss": "https://idp.example.org/realms/test",
            "aud": "my client",
            "exp": 1000,
            "nonce": "my nonce",
        })
    }

    fn validate(claims: serde_json::Value) -> Result<()> {
        let claims: Claims = serde_json::from_value(claims).unwrap();
        validate_claims(
            &claims,
            "https://idp.example.org/realms/test",
            "my client",
            "my nonce",
            999,
        )
    }

    fn cookie_headers(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, value.parse().unwrap());
        headers
    }

    #[test]
    fn test_validate_claims() {
        assert!(validate(create_claims()).is_ok());

        let mut claims = create_claims();
        claims["aud"] = json!(["other client", "my client"]);
        claims["azp"] = json!("my client");
        assert!(validate(claims).is_ok());

        let invalid = [
            ("iss", json!("https://evil.example.org")),
            ("aud", json!("other client")),
            ("aud", json!(["other client"])),
            ("azp", json!("other client")),
            ("exp", json!(999)),
            ("nonce", json!("other nonce")),
            ("nonce", json!(null)),
        ];
        for (name, value) in invalid {
            let mut claims = create_claims();
            claims[name] = value;
            assert!(validate(claims).is_err(), "{name} was accepted");
        }
    }

    #[test]
    fn test_decode_claims() {
        let payload = BASE64_URL_SAFE_NO_PAD.encode(create_claims().to_string());
        let claims = decode_claims(&format!("header.{payload}.signature")).unwrap();
        assert_eq!(claims.nonce.as_deref(), Some("my nonce"));

        assert!(decode_claims("no jwt").is_err());
        assert!(decode_claims("header.no base64!.signature").is_err());
    }

    #[test]
    fn test_parse_redirect_uri() {
        let (base_path, secure) =
            parse_redirect_uri("https://dmarc.example.org/oidc/callback").unwrap();
        assert_eq!(base_path, "/");
        assert!(secure);

        let (base_path, secure) =
            parse_redirect_uri("http://localhost:8080/sub/oidc/callback").unwrap();
        assert_eq!(base_path, "/sub/");
        assert!(!secure);

        assert!(parse_redirect_uri("https://dmarc.example.org/").is_err());
        assert!(parse_redirect_uri("https://dmarc.example.org/xoidc/callback").is_err());
    }

    #[test]
    fn test_get_cookie() {
        let headers = cookie_headers("a=1; dmarc_session=abc; b=2");
        assert_eq!(get_cookie(&headers, SESSION_COOKIE), Some("abc"));
        assert_eq!(get_cookie(&headers, STATE_COOKIE), None);
    }

    #[test]
    fn test_sessions() {
        let oidc = create_oidc();
        let now = Instant::now();
        {
            let mut sessions = oidc.sessions.lock().unwrap();
            sessions.insert(String::from("valid"), now + SESSION_LIFETIME);
            sessions.insert(String::from("expired"), now);
        }
        assert!(oidc.has_session(&cookie_headers("dmarc_session=valid")));
        assert!(!oidc.has_session(&cookie_headers("dmarc_session=expired")));
        assert!(!oidc.has_session(&cookie_headers("dmarc_session=unknown")));
        assert!(!oidc.has_session(&HeaderMap::new()));
    }

    #[test]
    fn test_start_login() {
        let oidc = create_oidc();
        let (url, state) = oidc.start_login().unwrap();
        assert!(url.starts_with("https://idp.example.org/auth?response_type=code"));
        assert!(url.contains("&client_id=my%20client&"));
        assert!(url.contains("&redirect_uri=https%3A%2F%2Fdmarc.example.org%2Foidc%2Fcallback&"));
        assert!(url.contains(&format!("&state={state}&")));

        // PKCE challenge must be the hash of the stored verifier
        let pending_logins = oidc.pending_logins.lock().unwrap();
        let login = pending_logins.get(&state).unwrap();
        let challenge = BASE64_URL_SAFE_NO_PAD.encode(Sha256::digest(&login.code_verifier));
        assert!(url.contains(&format!("&code_challenge={challenge}&")));
        assert!(url.contains(&format!("&nonce={}&", login.nonce)));
    }

    #[tokio::test]
    async fn test_finish_login_rejects_foreign_state() {
        let oidc = create_oidc();
        let (_, state) = oidc.start_login().unwrap();
        let params = || CallbackParams {
            code: Some(String::from("code")),
            state: Some(state.clone()),
            error: None,
        };

        // Without the matching state cookie the pending login must stay untouched
        let result = oidc.finish_login(&HeaderMap::new(), params()).await;
        assert!(result.is_err());
        let headers = cookie_headers("dmarc_oidc_state=other");
        assert!(oidc.finish_login(&headers, params()).await.is_err());
        assert_eq!(oidc.pending_logins.lock().unwrap().len(), 1);
        assert!(oidc.sessions.lock().unwrap().is_empty());
    }
}
