use anyhow::{Context, Result, bail, ensure};
use dns_protocol::{Flags, Message, Question, ResourceRecord, ResourceType};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::AtomicU16;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

pub struct DnsClient {
    server: SocketAddr,
    next_id: AtomicU16,
    timeout: Duration,
}

impl DnsClient {
    pub fn new(server: SocketAddr, timeout: Duration) -> Self {
        Self {
            server,
            next_id: AtomicU16::new(1),
            timeout,
        }
    }

    pub async fn host_from_ip(&self, ip: IpAddr) -> Result<Option<String>> {
        // Create a unique ID for the query
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Create the query string
        let query = match ip {
            IpAddr::V4(addr) => Self::ipv4_query(addr),
            IpAddr::V6(addr) => Self::ipv6_query(addr),
        };

        // Create a message
        let mut questions = [Question::new(query.as_str(), ResourceType::Ptr, 1)];
        let message = Message::new(
            id,
            Flags::standard_query(),
            &mut questions,
            &mut [],
            &mut [],
            &mut [],
        );

        // Send message and receive DNS response data
        let response = self
            .send_message_receive_udp_data(&message)
            .await
            .context("Failed to send/receive DNS data")?;

        // Parse the data as a message
        let mut answers = [ResourceRecord::default(); 1];
        let mut authorities = [ResourceRecord::default(); 1];
        let mut additionals = [ResourceRecord::default(); 1];
        let message = Message::read(
            &response,
            &mut questions,
            &mut answers,
            &mut authorities,
            &mut additionals,
        )
        .context("Failed to read DNS message")?;

        // Make sure we got the right answer
        ensure!(
            message.id() == id,
            "Received response with mismatched ID: expected {}, got {}",
            id,
            message.id()
        );

        // Read the answer from the message
        let Some(answer) = message.answers().first() else {
            return Ok(None);
        };

        // Check the answer type
        if answer.ty() != ResourceType::Ptr {
            bail!("Wrong answer type: {:?}", answer.ty())
        }

        // Parse the DNS name
        Ok(Some(
            parse_dns_name(answer.data()).context("Failed to parse DNS name")?,
        ))
    }

    fn ipv4_query(ip: Ipv4Addr) -> String {
        // Reverse the octets for PTR query
        let octets = ip.octets();
        format!(
            "{}.{}.{}.{}.in-addr.arpa",
            octets[3], octets[2], octets[1], octets[0]
        )
    }

    fn ipv6_query(ip: Ipv6Addr) -> String {
        // Get the 8bit segments of the IPv6 address
        let octets = ip.octets();

        // Convert each nibble to hex, reverse order, join with dots
        let mut nibbles = Vec::with_capacity(32);
        for &octet in &octets {
            nibbles.push(format!("{:x}", (octet & 0xF0) >> 4));
            nibbles.push(format!("{:x}", octet & 0x0F));
        }
        nibbles.reverse();
        format!("{}.ip6.arpa", nibbles.join("."))
    }

    async fn send_message_receive_udp_data(&self, message: &Message<'_, '_>) -> Result<Vec<u8>> {
        // Serialize the message into a buffer
        let mut buf = vec![0; 1024];
        assert!(message.space_needed() <= buf.len());
        let len = message
            .write(&mut buf)
            .context("Failed to serialize DNS message")?;

        // Create a UDP socket
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .context("Failed to bind UDP socket")?;

        // Send the data
        socket
            .send_to(&buf[..len], self.server)
            .await
            .context("Failed to send data")?;

        // Read response data from the socket
        let mut response = vec![0; 1024];
        let len = timeout(self.timeout, socket.recv(&mut response))
            .await
            .context("Timeout while reading response")?
            .context("Failed to read response")?;
        response.truncate(len);

        Ok(response)
    }
}

// Parse a DNS name from DNS label format (RFC 1035)
fn parse_dns_name(data: &[u8]) -> Result<String> {
    let mut labels = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let len = data[i] as usize;
        if len == 0 {
            break;
        }
        i += 1;
        if i + len > data.len() {
            bail!("Label length out of bounds");
        }
        let label = data[i..i + len].to_owned();
        let parsed_label = String::from_utf8(label).context("Failed to parse segment as UTF8")?;
        labels.push(parsed_label);
        i += len;
    }
    Ok(labels.join("."))
}
