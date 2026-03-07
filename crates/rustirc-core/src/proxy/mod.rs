//! Proxy connection support for IRC
//!
//! Provides SOCKS5 and HTTP CONNECT proxy implementations for tunneling
//! IRC connections through proxy servers.

pub mod http;
pub mod socks5;

use async_trait::async_trait;
use tokio::net::TcpStream;

use crate::config::ProxyConfig;
use crate::error::Result;

pub use http::HttpProxy;
pub use socks5::Socks5Proxy;

/// Trait for proxy connectors that establish a tunneled TCP connection
/// to a target host through a proxy server.
#[async_trait]
pub trait ProxyConnector: Send + Sync {
    /// Connect to the target address and port through the proxy.
    ///
    /// Returns a `TcpStream` that is connected to the target through the
    /// proxy, ready for normal read/write operations.
    async fn connect(&self, target_addr: &str, target_port: u16) -> Result<TcpStream>;
}

/// Create a `ProxyConnector` from a `ProxyConfig`.
pub fn from_config(config: &ProxyConfig) -> Box<dyn ProxyConnector> {
    match config.proxy_type {
        crate::config::ProxyType::Socks5 => Box::new(Socks5Proxy::new(
            config.address.clone(),
            config.port,
            config.username.clone(),
            config.password.clone(),
        )),
        crate::config::ProxyType::HttpConnect => Box::new(HttpProxy::new(
            config.address.clone(),
            config.port,
            config.username.clone(),
            config.password.clone(),
        )),
    }
}
