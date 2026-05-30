//! Proxy support for agent connections
//!
//! Supports SOCKS5 and HTTP CONNECT proxies

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::config::{HttpAuth, ProxyConfig, Socks5Auth};

/// Result of proxy connection
pub struct ProxyStream {
    pub stream: TcpStream,
    pub target_host: String,
    pub target_port: u16,
}

/// Parse a URL into host and port
fn parse_url(url: &str) -> Result<(String, u16), String> {
    let url = url
        .strip_prefix("ws://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    let (host_port, _) = url.split_once('/').unwrap_or((url, ""));
    if host_port.contains(':') {
        let parts: Vec<&str> = host_port.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid host:port format: {}", host_port));
        }
        let host = parts[0].to_string();
        let port: u16 = parts[1].parse()
            .map_err(|_| format!("Invalid port in URL: {}", host_port))?;
        Ok((host, port))
    } else {
        Ok((host_port.to_string(), 80))
    }
}

/// Connect through a SOCKS5 proxy
pub async fn connect_via_socks5(
    proxy_host: &str,
    proxy_port: u16,
    target_host: &str,
    target_port: u16,
    auth: Option<&Socks5Auth>,
) -> Result<ProxyStream, String> {
    let addr = format!("{}:{}", proxy_host, proxy_port);
    let mut stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("Failed to connect to SOCKS5 proxy {}: {}", addr, e))?;

    // SOCKS5 greeting
    stream.write_all(&[0x05]).await
        .map_err(|e| format!("Failed to send SOCKS5 greeting: {}", e))?;

    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf).await
        .map_err(|e| format!("Failed to read SOCKS5 response: {}", e))?;

    if buf[0] != 0x05 {
        return Err(format!("SOCKS5 version mismatch: {}", buf[0]));
    }

    // Authentication method
    let methods = if auth.is_some() {
        vec![0x02, 0x00] // username/password, no auth
    } else {
        vec![0x00] // no auth
    };
    stream.write_all(&[0x05, methods.len() as u8]).await
        .map_err(|e| format!("Failed to send auth methods: {}", e))?;
    stream.write_all(&methods).await
        .map_err(|e| format!("Failed to send auth methods: {}", e))?;

    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf).await
        .map_err(|e| format!("Failed to read auth response: {}", e))?;

    let method = buf[1];
    if method == 0x02 {
        // Username/password auth
        if let Some(auth) = auth {
            let username = &auth.username;
            let password = &auth.password;

            let mut auth_pkt = vec![0x01]; // version
            auth_pkt.push(username.len() as u8);
            auth_pkt.extend_from_slice(username.as_bytes());
            auth_pkt.push(password.len() as u8);
            auth_pkt.extend_from_slice(password.as_bytes());

            stream.write_all(&auth_pkt).await
                .map_err(|e| format!("Failed to send auth: {}", e))?;

            let mut buf = [0u8; 2];
            stream.read_exact(&mut buf).await
                .map_err(|e| format!("Failed to read auth response: {}", e))?;

            if buf[1] != 0x00 {
                return Err("SOCKS5 authentication failed".to_string());
            }
        }
    } else if method == 0xff {
        return Err("No acceptable authentication method".to_string());
    }

    // Connect request (IPv4)
    let mut request = vec![
        0x05, // version
        0x01, // connect
        0x00, // reserved
        0x01, // IPv4
    ];
    // Add IPv4 address (4 bytes)
    request.extend_from_slice(&[0, 0, 0, 0]);
    // Add port (2 bytes big-endian)
    request.extend_from_slice(&(target_port.to_be_bytes()));

    // Resolve hostname first (simplified - just use as-is)
    // In production, you'd want to handle DNS resolution properly
    stream.write_all(&request).await
        .map_err(|e| format!("Failed to send connect request: {}", e))?;

    let mut buf = [0u8; 10];
    stream.read_exact(&mut buf).await
        .map_err(|e| format!("Failed to read connect response: {}", e))?;

    if buf[1] != 0x00 {
        let error = match buf[1] {
            0x01 => "General SOCKS server failure",
            0x02 => "Connection not allowed by ruleset",
            0x03 => "Network unreachable",
            0x04 => "Host unreachable",
            0x05 => "Connection refused",
            0x06 => "TTL expired",
            0x07 => "Command not supported",
            0x08 => "Address type not supported",
            _ => "Unknown error",
        };
        return Err(format!("SOCKS5 connection failed: {} (code: {})", error, buf[1]));
    }

    Ok(ProxyStream {
        stream,
        target_host: target_host.to_string(),
        target_port,
    })
}

/// Connect through an HTTP CONNECT proxy
pub async fn connect_via_http_proxy(
    proxy_host: &str,
    proxy_port: u16,
    target_host: &str,
    target_port: u16,
    auth: Option<&HttpAuth>,
) -> Result<ProxyStream, String> {
    let addr = format!("{}:{}", proxy_host, proxy_port);
    let mut stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("Failed to connect to HTTP proxy {}: {}", addr, e))?;

    // Build CONNECT request
    let auth_header = if let Some(auth) = auth {
        let credentials = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:{}", auth.username, auth.password),
        );
        format!("Proxy-Authorization: Basic {}\r\n", credentials)
    } else {
        String::new()
    };

    let connect_request = format!(
        "CONNECT {}:{} HTTP/1.1\r\n\
         Host: {}:{}\r\n\
         {}\
         Connection: keep-alive\r\n\
         \r\n",
        target_host, target_port, target_host, target_port, auth_header
    );

    stream.write_all(connect_request.as_bytes()).await
        .map_err(|e| format!("Failed to send CONNECT request: {}", e))?;

    // Read response
    let mut response = Vec::new();
    let mut buf = [0u8; 1024];
    loop {
        let n = stream.read(&mut buf).await
            .map_err(|e| format!("Failed to read proxy response: {}", e))?;
        if n == 0 {
            return Err("Proxy closed connection".to_string());
        }
        response.extend_from_slice(&buf[..n]);

        // Check if we've received complete headers
        if response.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }

    let response_str = String::from_utf8_lossy(&response);

    // Check status code
    if !response_str.contains("200") && !response_str.contains("Connected") {
        // Extract error message
        let error = response_str
            .lines()
            .find(|l| l.starts_with("HTTP/"))
            .unwrap_or("Unknown error")
            .to_string();
        return Err(format!("HTTP proxy CONNECT failed: {}", error));
    }

    Ok(ProxyStream {
        stream,
        target_host: target_host.to_string(),
        target_port,
    })
}

/// Connect to a target through the configured proxy
pub async fn connect_proxy(
    proxy: &ProxyConfig,
    target_url: &str,
) -> Result<ProxyStream, String> {
    let (target_host, target_port) = parse_url(target_url)?;

    match proxy {
        ProxyConfig::None => {
            // Direct connection - not through this function
            Err("Use direct TcpStream connection for no proxy".to_string())
        }
        ProxyConfig::Socks5 { host, port, auth } => {
            connect_via_socks5(host, *port, &target_host, target_port, auth.as_ref()).await
        }
        ProxyConfig::Http { host, port, auth } => {
            connect_via_http_proxy(host, *port, &target_host, target_port, auth.as_ref()).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        let (host, port) = parse_url("ws://example.com:8080/path").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 8080);

        let (host, port) = parse_url("example.com:9000").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 9000);
    }
}
