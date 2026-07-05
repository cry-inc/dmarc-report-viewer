use anyhow::{Context, Result, bail, ensure};
use dns_protocol::{Flags, Message, Question, ResourceRecord, ResourceType};
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicU16, Ordering};
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
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

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
        let host_name =
            parse_dns_name(&response, answer.data()).context("Failed to parse DNS name")?;

        Ok(Some(host_name))
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

        // Create a UDP socket bound to the same address family as the server.
        let bind_addr = if self.server.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        };
        let socket = UdpSocket::bind(bind_addr)
            .await
            .context("Failed to bind UDP socket")?;

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

// Parse a DNS name
fn parse_dns_name(message: &[u8], data: &[u8]) -> Result<String> {
    let start = data.as_ptr() as usize - message.as_ptr() as usize;
    let labels = parse_dns_name_at_offset(message, start, &mut HashSet::new())?;
    Ok(labels.join("."))
}

fn parse_dns_name_at_offset(
    message: &[u8],
    mut cursor: usize,
    visited: &mut std::collections::HashSet<usize>,
) -> Result<Vec<String>> {
    let mut labels = Vec::new();

    loop {
        if cursor >= message.len() {
            bail!("Label length out of bounds");
        }

        let len = message[cursor] as usize;
        if len == 0 {
            break;
        }

        if len & 0xC0 == 0xC0 {
            if cursor + 1 >= message.len() {
                bail!("Compression pointer out of bounds");
            }

            let ptr = (((len & 0x3F) as u16) << 8) | message[cursor + 1] as u16;
            let ptr = ptr as usize;
            if !visited.insert(ptr) {
                bail!("Compression pointer loop detected");
            }

            let mut tail_labels = parse_dns_name_at_offset(message, ptr, visited)?;
            labels.append(&mut tail_labels);
            break;
        }

        if cursor + 1 + len > message.len() {
            bail!("Label length out of bounds");
        }

        let label_bytes = &message[cursor + 1..cursor + 1 + len];
        let parsed_label =
            std::str::from_utf8(label_bytes).context("Failed to parse segment as UTF8")?;
        labels.push(parsed_label.to_string());
        cursor += 1 + len;
    }

    Ok(labels)
}

#[cfg(test)]
mod tests {
    use super::parse_dns_name;

    #[test]
    fn parses_dns_wire_format_names() {
        let data = [
            3, b'w', b'w', b'w', 7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o', b'm',
            0,
        ];
        assert_eq!(parse_dns_name(&data, &data).unwrap(), "www.example.com");
    }

    #[test]
    fn parses_compressed_dns_wire_format_names() {
        let message = [7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 0, 0xC0, 0x00];
        let data = &message[9..11];
        assert_eq!(parse_dns_name(&message, data).unwrap(), "example");
    }

    #[test]
    fn parses_compressed_tail_labels() {
        let message = [
            3, b'w', b'w', b'w', 0xC0, 0x06, /* pointer to offset 6 */
            7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 3, b'c', b'o', b'm', 0,
        ];
        let data = &message[0..6];
        assert_eq!(parse_dns_name(&message, data).unwrap(), "www.example.com");
    }

    #[test]
    fn rejects_compression_pointer_loop() {
        let message = [0xC0, 0x02, 0xC0, 0x00];
        let data = &message[0..2];
        let err = parse_dns_name(&message, data).unwrap_err();
        assert!(
            err.to_string()
                .contains("Compression pointer loop detected")
        );
    }
}
