//! HTTP CONNECT proxy support
//!
//! Implements HTTP CONNECT tunneling for establishing TCP connections
//! through an HTTP proxy. Supports optional Basic authentication via
//! the Proxy-Authorization header.
//!
//! See: RFC 7231 Section 4.3.6 (CONNECT), RFC 7235 (HTTP Authentication)

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use super::ProxyConnector;
use crate::error::{Error, Result};

/// An HTTP CONNECT proxy connector.
///
/// Establishes TCP connections through an HTTP proxy using the CONNECT
/// method. Optionally provides Basic authentication credentials.
#[derive(Debug, Clone)]
pub struct HttpProxy {
    /// The proxy server address (hostname or IP).
    proxy_addr: String,
    /// The proxy server port.
    proxy_port: u16,
    /// Optional username for Basic auth.
    username: Option<String>,
    /// Optional password for Basic auth.
    password: Option<String>,
}

impl HttpProxy {
    /// Create a new HTTP CONNECT proxy configuration.
    pub fn new(
        proxy_addr: String,
        proxy_port: u16,
        username: Option<String>,
        password: Option<String>,
    ) -> Self {
        Self {
            proxy_addr,
            proxy_port,
            username,
            password,
        }
    }

    /// Return the proxy address in `host:port` format.
    pub fn proxy_address(&self) -> String {
        format!("{}:{}", self.proxy_addr, self.proxy_port)
    }

    /// Build the HTTP CONNECT request for the target.
    fn build_connect_request(&self, target_addr: &str, target_port: u16) -> String {
        let mut request = format!(
            "CONNECT {target_addr}:{target_port} HTTP/1.1\r\n\
             Host: {target_addr}:{target_port}\r\n"
        );

        // Add Basic auth header if credentials are provided
        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            use base64::Engine;
            let credentials = format!("{user}:{pass}");
            let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
            request.push_str(&format!("Proxy-Authorization: Basic {encoded}\r\n"));
        }

        request.push_str("\r\n");
        request
    }
}

#[async_trait]
impl ProxyConnector for HttpProxy {
    /// Connect to the target through the HTTP CONNECT proxy.
    ///
    /// Establishes a TCP connection to the proxy, sends a CONNECT request,
    /// reads the response, and returns the tunneled connection on success.
    async fn connect(&self, target_addr: &str, target_port: u16) -> Result<TcpStream> {
        let proxy_addr = self.proxy_address();

        // Connect to the proxy server
        let stream = TcpStream::connect(&proxy_addr).await.map_err(|e| {
            Error::Connection(format!(
                "Failed to connect to HTTP proxy at {proxy_addr}: {e}"
            ))
        })?;

        // Send the CONNECT request
        let connect_request = self.build_connect_request(target_addr, target_port);
        let (reader, mut writer) = tokio::io::split(stream);
        writer
            .write_all(connect_request.as_bytes())
            .await
            .map_err(|e| {
                Error::Connection(format!(
                    "Failed to send CONNECT request to proxy {proxy_addr}: {e}"
                ))
            })?;

        // Read the response status line
        let mut buf_reader = BufReader::new(reader);
        let mut status_line = String::new();
        buf_reader.read_line(&mut status_line).await.map_err(|e| {
            Error::Connection(format!(
                "Failed to read proxy response from {proxy_addr}: {e}"
            ))
        })?;

        // Parse the HTTP status code
        let status_code = parse_http_status(&status_line).ok_or_else(|| {
            Error::Connection(format!(
                "Invalid HTTP response from proxy {proxy_addr}: {status_line}"
            ))
        })?;

        if status_code != 200 {
            return Err(Error::Connection(format!(
                "HTTP CONNECT to {target_addr}:{target_port} via {proxy_addr} \
                 failed with status {status_code}: {status_line}"
            )));
        }

        // Read and discard remaining headers (until blank line)
        loop {
            let mut header_line = String::new();
            let n = buf_reader.read_line(&mut header_line).await.map_err(|e| {
                Error::Connection(format!(
                    "Failed to read proxy headers from {proxy_addr}: {e}"
                ))
            })?;

            if n == 0 || header_line.trim().is_empty() {
                break;
            }
        }

        // Reassemble the stream from the split halves
        let stream = buf_reader.into_inner().unsplit(writer);

        Ok(stream)
    }
}

/// Parse the HTTP status code from a status line like "HTTP/1.1 200 OK\r\n".
fn parse_http_status(status_line: &str) -> Option<u16> {
    let parts: Vec<&str> = status_line.trim().splitn(3, ' ').collect();
    if parts.len() >= 2 && parts[0].starts_with("HTTP/") {
        parts[1].parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_proxy_creation() {
        let proxy = HttpProxy::new(
            "proxy.example.com".to_string(),
            8080,
            Some("user".to_string()),
            Some("secret".to_string()),
        );

        assert_eq!(proxy.proxy_address(), "proxy.example.com:8080");
        assert_eq!(proxy.username, Some("user".to_string()));
        assert_eq!(proxy.password, Some("secret".to_string()));
    }

    #[test]
    fn test_connect_request_no_auth() {
        let proxy = HttpProxy::new("proxy.local".to_string(), 3128, None, None);
        let request = proxy.build_connect_request("irc.libera.chat", 6697);

        assert!(request.starts_with("CONNECT irc.libera.chat:6697 HTTP/1.1\r\n"));
        assert!(request.contains("Host: irc.libera.chat:6697\r\n"));
        assert!(!request.contains("Proxy-Authorization"));
        assert!(request.ends_with("\r\n\r\n"));
    }

    #[test]
    fn test_connect_request_with_auth() {
        let proxy = HttpProxy::new(
            "proxy.local".to_string(),
            3128,
            Some("alice".to_string()),
            Some("password123".to_string()),
        );
        let request = proxy.build_connect_request("irc.example.com", 6667);

        assert!(request.contains("CONNECT irc.example.com:6667 HTTP/1.1\r\n"));
        assert!(request.contains("Proxy-Authorization: Basic "));

        // Verify the Base64 encoding of "alice:password123"
        use base64::Engine;
        let expected =
            base64::engine::general_purpose::STANDARD.encode("alice:password123".as_bytes());
        assert!(
            request.contains(&expected),
            "Request should contain base64 of 'alice:password123': {expected}"
        );
    }

    #[test]
    fn test_parse_http_status() {
        assert_eq!(
            parse_http_status("HTTP/1.1 200 Connection established\r\n"),
            Some(200)
        );
        assert_eq!(
            parse_http_status("HTTP/1.0 407 Proxy Authentication Required\r\n"),
            Some(407)
        );
        assert_eq!(parse_http_status("HTTP/1.1 403 Forbidden"), Some(403));
        assert_eq!(parse_http_status("HTTP/1.1 200"), Some(200));
        assert_eq!(parse_http_status("INVALID RESPONSE"), None);
        assert_eq!(parse_http_status(""), None);
    }

    #[tokio::test]
    async fn test_http_connect_failure() {
        // Attempt to connect to a non-existent proxy - should produce a
        // meaningful error rather than a panic.
        let proxy = HttpProxy::new("127.0.0.1".to_string(), 1, None, None);
        let result = proxy.connect("irc.example.com", 6667).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("HTTP proxy"),
            "Error should mention HTTP proxy: {msg}"
        );
    }
}
