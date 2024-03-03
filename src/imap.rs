use crate::config::Configuration;
use anyhow::{Context, Result};
use imap::Client;
use rustls_connector::RustlsConnector;
use std::net::TcpStream;

pub fn get_mails(config: &Configuration) -> Result<Vec<Vec<u8>>> {
    let mut mails = Vec::new();
    let addr = (config.imap_host.as_str(), config.imap_port);
    let stream = TcpStream::connect(addr).context("Failed to connect to IMAP server")?;
    let connector =
        RustlsConnector::new_with_native_certs().expect("Failed to create Rust TLS connector");
    let tls_stream = connector
        .connect(&config.imap_host, stream)
        .context("Failed to set up TLS stream")?;
    let client = Client::new(tls_stream);
    let mut session = client
        .login(&config.imap_user, &config.imap_password)
        .expect("Failed to log in");
    let mailbox = session.select("INBOX").context("Failed to select inbox")?;
    if mailbox.exists > 0 {
        let sequence = format!("1:{}", mailbox.exists);
        let messages = session
            .fetch(sequence, "RFC822")
            .context("Failed to fetch first message")?;
        for message in messages.iter() {
            let body = message.body().context("Message did not have a body!")?;
            mails.push(body.to_vec());
        }
    }
    session.logout().context("Failed to log out")?;
    Ok(mails)
}
