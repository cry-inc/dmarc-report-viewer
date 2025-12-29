use axum::serve::Listener;
use futures::StreamExt;
use rustls_acme::caches::DirCache;
use rustls_acme::{AcmeConfig, is_tls_alpn_challenge};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::io::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::LazyConfigAcceptor;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::server::TlsStream;
use tracing::{error, info, warn};

pub struct AcmeListener {
    tcp_listener: TcpListener,
    challenge_config: Arc<ServerConfig>,
    default_config: Arc<ServerConfig>,
}

impl AcmeListener {
    pub fn new(
        tcp_listener: TcpListener,
        acme_domain: String,
        acme_contact: String,
        acme_cache_dir: PathBuf,
    ) -> anyhow::Result<Self> {
        // Set up cert caching dir
        let acme_cache = DirCache::new(acme_cache_dir);

        // Set up ACME client
        let mut acme_state = AcmeConfig::new([acme_domain])
            .contact([acme_contact])
            .cache_option(Some(acme_cache))
            .directory_lets_encrypt(true)
            .state();

        // Prepare the configurations for incoming connections
        let challenge_config = acme_state.challenge_rustls_config();
        let default_config = acme_state.default_rustls_config();

        // Background task to handle ACME client state
        tokio::spawn(async move {
            while let Some(result) = acme_state.next().await {
                match result {
                    Ok(ok) => info!("ACME event: {ok:?}"),
                    Err(err) => error!("ACME error: {err:?}"),
                }
            }
        });

        // Create listener to accept incomding connections
        Ok(Self {
            tcp_listener,
            challenge_config,
            default_config,
        })
    }
}

impl Listener for AcmeListener {
    type Io = TlsStream<TcpStream>;
    type Addr = SocketAddr;

    fn local_addr(&self) -> Result<Self::Addr> {
        self.tcp_listener.local_addr()
    }

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            // Wait for next TCP connection
            let result = self.tcp_listener.accept().await;
            let (stream, addr) = match result {
                Ok(tuple) => tuple,
                Err(err) => {
                    warn!("Failed to accept TCP stream: {err}");
                    continue;
                }
            };

            // Initiate TLS handshake
            let result = LazyConfigAcceptor::new(Default::default(), stream).await;
            let handshake = match result {
                Ok(hs) => hs,
                Err(err) => {
                    warn!("Failed to initiate TLS handshake: {err}");
                    continue;
                }
            };

            if is_tls_alpn_challenge(&handshake.client_hello()) {
                // Handle ACME challenges
                info!("Received TLS-ALPN-01 validation request");
                let config = self.challenge_config.clone();
                let result = handshake.into_stream(config).await;
                let mut tls = match result {
                    Ok(tls) => tls,
                    Err(err) => {
                        warn!("Failed to handle TLS-ALPN-01 validation request: {err}");
                        continue;
                    }
                };
                if let Err(err) = tls.shutdown().await {
                    warn!("Failed to shut down TLS connection for validation request: {err}");
                }
            } else {
                // Handle normal incoming connection
                let config = self.default_config.clone();
                let result = handshake.into_stream(config).await;
                match result {
                    Ok(tls) => return (tls, addr),
                    Err(err) => warn!("Failed to create TLS stream: {err}"),
                };
            }
        }
    }
}
