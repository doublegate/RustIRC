//! SOCKS5 proxy connection support
//!
//! Implements SOCKS5 proxy tunneling using the `tokio-socks` crate,
//! supporting both unauthenticated and username/password authenticated
//! connections.
//!
//! See: RFC 1928 (SOCKS5), RFC 1929 (SOCKS5 Username/Password Auth)

use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;

use super::ProxyConnector;
use crate::error::{Error, Result};

/// A SOCKS5 proxy connector.
///
/// Establishes TCP connections through a SOCKS5 proxy server, optionally
/// using username/password authentication.
#[derive(Debug, Clone)]
pub struct Socks5Proxy {
    /// The proxy server address (hostname or IP).
    proxy_addr: String,
    /// The proxy server port.
    proxy_port: u16,
    /// Optional username for proxy authentication.
    username: Option<String>,
    /// Optional password for proxy authentication.
    password: Option<String>,
}

impl Socks5Proxy {
    /// Create a new SOCKS5 proxy configuration.
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
}

#[async_trait]
impl ProxyConnector for Socks5Proxy {
    /// Connect to the target through the SOCKS5 proxy.
    ///
    /// If username and password are both provided, uses authenticated
    /// SOCKS5. Otherwise, uses unauthenticated mode.
    async fn connect(&self, target_addr: &str, target_port: u16) -> Result<TcpStream> {
        let proxy_addr = self.proxy_address();
        let target = (target_addr, target_port);

        let stream = match (&self.username, &self.password) {
            (Some(user), Some(pass)) => Socks5Stream::connect_with_password(
                proxy_addr.as_str(),
                target,
                user.as_str(),
                pass.as_str(),
            )
            .await
            .map_err(|e| {
                Error::Connection(format!(
                    "SOCKS5 authenticated connection to {target_addr}:{target_port} \
                         via {proxy_addr} failed: {e}"
                ))
            })?,
            _ => Socks5Stream::connect(proxy_addr.as_str(), target)
                .await
                .map_err(|e| {
                    Error::Connection(format!(
                        "SOCKS5 connection to {target_addr}:{target_port} \
                             via {proxy_addr} failed: {e}"
                    ))
                })?,
        };

        Ok(stream.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socks5_proxy_creation() {
        let proxy = Socks5Proxy::new(
            "proxy.example.com".to_string(),
            1080,
            Some("user".to_string()),
            Some("pass".to_string()),
        );

        assert_eq!(proxy.proxy_address(), "proxy.example.com:1080");
        assert_eq!(proxy.username, Some("user".to_string()));
        assert_eq!(proxy.password, Some("pass".to_string()));
    }

    #[test]
    fn test_socks5_proxy_no_auth() {
        let proxy = Socks5Proxy::new("127.0.0.1".to_string(), 9050, None, None);

        assert_eq!(proxy.proxy_address(), "127.0.0.1:9050");
        assert!(proxy.username.is_none());
        assert!(proxy.password.is_none());
    }

    #[tokio::test]
    async fn test_socks5_connect_failure() {
        // Attempt to connect to a non-existent proxy - should produce a
        // meaningful error rather than a panic.
        let proxy = Socks5Proxy::new("127.0.0.1".to_string(), 1, None, None);
        let result = proxy.connect("irc.example.com", 6667).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("SOCKS5 connection"),
            "Error should mention SOCKS5: {msg}"
        );
    }
}
