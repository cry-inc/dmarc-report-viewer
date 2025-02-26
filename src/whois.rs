// This file contains a minimal whois client for IPs.
// It will not work for domains.
// Its a heavily modified and simplified version of an existing library.
// See https://github.com/magiclen/whois-rust for the original code!

use anyhow::{bail, Context, Result};
use regex::Regex;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

pub struct WhoIsIp {
    regex: Regex,
    server: Server,
    timeout: Duration,
    max_follows: u8,
}

impl Default for WhoIsIp {
    fn default() -> Self {
        Self {
            server: Server::default(),
            regex: Regex::new(r"(ReferralServer|Registrar Whois|Whois Server|WHOIS Server|Registrar WHOIS Server):[^\S\n]*(r?whois://)?(.*)").expect("Failed to cosntruct RegEx"),
            timeout: Duration::from_secs(10),
            max_follows: 3,
        }
    }
}

impl WhoIsIp {
    async fn get_tcp_stream(&self, addr: &str) -> Result<TcpStream> {
        timeout(self.timeout, TcpStream::connect(addr))
            .await
            .context("TCP connect timed out")?
            .context("TCP connect failed")
    }

    async fn lookup_once(&self, ip: &str, server: &Server) -> Result<AddrTextPair> {
        let server_addr = format!("{}:{}", server.host, server.port);
        let mut client = self
            .get_tcp_stream(&server_addr)
            .await
            .context("Failed to get TCP stream")?;
        let query = server.query.replace("$addr", ip);
        timeout(self.timeout, client.write_all(query.as_bytes()))
            .await
            .context("Sending query timed out")?
            .context("Failed to send query")?;
        timeout(self.timeout, client.flush())
            .await
            .context("Flushing query timed out")?
            .context("Failed to flush query")?;
        let mut result = String::new();
        timeout(self.timeout, client.read_to_string(&mut result))
            .await
            .context("Reading response timed out")?
            .context("Failed to read response")?;
        Ok(AddrTextPair {
            server_addr,
            text: result,
        })
    }

    async fn lookup_iterative(&self, ip: &str, server: &Server, mut follow: u8) -> Result<String> {
        let mut result = self
            .lookup_once(ip, server)
            .await
            .context("Initial whois query failed")?;
        while follow > 0 {
            if let Some(captures) = self.regex.captures(&result.text) {
                if let Some(addr) = captures.get(3) {
                    let addr = addr.as_str();
                    if addr.ne(&result.server_addr) {
                        let server =
                            Server::from_str(addr).context("Failed to parse server address")?;
                        result = self
                            .lookup_once(ip, &server)
                            .await
                            .context("Secondary whois query failed")?;
                        follow -= 1;
                        continue;
                    }
                }
            }
            break;
        }
        Ok(result.text)
    }

    pub async fn lookup(&self, ip: &str) -> Result<String> {
        self.lookup_iterative(ip, &self.server, self.max_follows)
            .await
    }
}

struct AddrTextPair {
    pub server_addr: String,
    pub text: String,
}

struct Server {
    pub host: String,
    pub port: u16,
    pub query: String,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            host: String::from("whois.arin.net"),
            port: 43,
            query: String::from("n + $addr\r\n"),
        }
    }
}

impl Server {
    fn from_str(value: &str) -> Result<Server> {
        let query = String::from("$addr\r\n");
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() == 1 {
            let host = parts[0].to_string();
            let port = 43;
            Ok(Server { host, query, port })
        } else if parts.len() >= 2 {
            let host = parts[0].to_string();
            let port: u16 = parts[1].parse().context("Failed to parse port")?;
            Ok(Server { host, query, port })
        } else {
            bail!("Cannot parse address, expected host[:port]")
        }
    }
}
