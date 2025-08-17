//! Error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Connection timeout")]
    ConnectionTimeout,
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    
    #[error("Invalid TLS name: {0}")]
    InvalidTlsName(String),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("TLS error: {0}")]
    TlsError(String),
    
    #[error("TLS config error: {0}")]
    TlsConfig(#[from] rustls::Error),
    
    #[error("Channel send error")]
    ChannelSend,
    
    #[error("Scripting error: {0}")]
    Scripting(String),
    
    #[error("Plugin error: {0}")]
    Plugin(String),
    
    #[error("State error: {0}")]
    State(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, Error>;