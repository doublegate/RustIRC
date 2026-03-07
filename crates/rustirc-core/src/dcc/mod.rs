//! DCC (Direct Client-to-Client) protocol implementation
//!
//! This module provides DCC support for RustIRC, enabling direct peer-to-peer
//! connections for file transfers (SEND/GET/RESUME) and private chats.
//!
//! # Architecture
//!
//! The DCC subsystem is managed by [`DccManager`], which tracks all active sessions
//! and coordinates lifecycle events. Individual sessions are handled by [`DccChat`]
//! (in [`chat`]) and [`DccTransfer`] (in [`transfer`]).
//!
//! # Protocol Overview
//!
//! DCC operates via CTCP messages exchanged over IRC to negotiate direct TCP
//! connections between clients. Once established, all data flows over the direct
//! connection without passing through the IRC server.

pub mod chat;
pub mod transfer;

use crate::config::DccConfig;
use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};

pub use chat::DccChat;
pub use transfer::{DccTransfer, TransferProgress};

/// Unique identifier for a DCC session.
pub type SessionId = u64;

/// Errors that can occur during DCC operations.
#[derive(Error, Debug)]
pub enum DccError {
    #[error("DCC is disabled in configuration")]
    Disabled,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Session not found: {0}")]
    SessionNotFound(SessionId),

    #[error("Session already exists: {0}")]
    SessionAlreadyExists(SessionId),

    #[error("File too large: {size} bytes exceeds limit of {limit} bytes")]
    FileTooLarge { size: u64, limit: u64 },

    #[error("Invalid port range: {start}-{end}")]
    InvalidPortRange { start: u16, end: u16 },

    #[error("No available port in range {start}-{end}")]
    NoAvailablePort { start: u16, end: u16 },

    #[error("Invalid DCC request: {0}")]
    InvalidRequest(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Transfer cancelled")]
    Cancelled,

    #[error("Resume not supported by peer")]
    ResumeNotSupported,

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Timeout waiting for connection")]
    Timeout,
}

/// Result type for DCC operations.
pub type DccResult<T> = std::result::Result<T, DccError>;

/// Direction of a DCC transfer or connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DccDirection {
    /// We are sending / hosting.
    Outgoing,
    /// We are receiving / connecting.
    Incoming,
}

impl fmt::Display for DccDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DccDirection::Outgoing => write!(f, "outgoing"),
            DccDirection::Incoming => write!(f, "incoming"),
        }
    }
}

/// Represents the type and metadata of an active DCC session.
#[derive(Debug, Clone)]
pub enum DccSession {
    /// A DCC CHAT session.
    Chat {
        /// Unique session identifier.
        id: SessionId,
        /// The remote IRC nick involved in this chat.
        peer_nick: String,
        /// Direction of the connection.
        direction: DccDirection,
        /// Remote IP address (if known).
        remote_addr: Option<IpAddr>,
        /// Remote port (if known).
        remote_port: Option<u16>,
        /// Whether the chat session is currently connected.
        connected: bool,
    },
    /// A DCC SEND (outgoing file transfer) session.
    Send {
        /// Unique session identifier.
        id: SessionId,
        /// The remote IRC nick receiving the file.
        peer_nick: String,
        /// Name of the file being sent.
        filename: String,
        /// Total size of the file in bytes.
        file_size: u64,
        /// Current transfer progress.
        progress: TransferProgress,
        /// Whether the transfer is currently active.
        active: bool,
    },
    /// A DCC RECEIVE (incoming file transfer) session.
    Receive {
        /// Unique session identifier.
        id: SessionId,
        /// The remote IRC nick sending the file.
        peer_nick: String,
        /// Name of the file being received.
        filename: String,
        /// Total size of the file in bytes.
        file_size: u64,
        /// Current transfer progress.
        progress: TransferProgress,
        /// Whether the transfer is currently active.
        active: bool,
    },
}

impl DccSession {
    /// Returns the session identifier.
    pub fn id(&self) -> SessionId {
        match self {
            DccSession::Chat { id, .. }
            | DccSession::Send { id, .. }
            | DccSession::Receive { id, .. } => *id,
        }
    }

    /// Returns the peer nickname.
    pub fn peer_nick(&self) -> &str {
        match self {
            DccSession::Chat { peer_nick, .. }
            | DccSession::Send { peer_nick, .. }
            | DccSession::Receive { peer_nick, .. } => peer_nick,
        }
    }

    /// Returns whether the session is currently active/connected.
    pub fn is_active(&self) -> bool {
        match self {
            DccSession::Chat { connected, .. } => *connected,
            DccSession::Send { active, .. } | DccSession::Receive { active, .. } => *active,
        }
    }
}

impl fmt::Display for DccSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DccSession::Chat {
                id,
                peer_nick,
                direction,
                connected,
                ..
            } => {
                let status = if *connected { "connected" } else { "pending" };
                write!(
                    f,
                    "[{id}] DCC CHAT with {peer_nick} ({direction}, {status})"
                )
            }
            DccSession::Send {
                id,
                peer_nick,
                filename,
                progress,
                active,
                ..
            } => {
                let status = if *active { "transferring" } else { "pending" };
                write!(
                    f,
                    "[{id}] DCC SEND {filename} to {peer_nick} ({status}, {:.1}%)",
                    progress.percentage
                )
            }
            DccSession::Receive {
                id,
                peer_nick,
                filename,
                progress,
                active,
                ..
            } => {
                let status = if *active { "transferring" } else { "pending" };
                write!(
                    f,
                    "[{id}] DCC RECV {filename} from {peer_nick} ({status}, {:.1}%)",
                    progress.percentage
                )
            }
        }
    }
}

/// Events emitted by the DCC subsystem to inform the rest of the application.
#[derive(Debug, Clone)]
pub enum DccEvent {
    /// A DCC offer was received from a remote peer.
    Offered {
        session_id: SessionId,
        peer_nick: String,
        /// A human-readable description of the offer.
        description: String,
    },
    /// A DCC session was accepted (locally or remotely).
    Accepted {
        session_id: SessionId,
        peer_nick: String,
    },
    /// Transfer progress update.
    Progress {
        session_id: SessionId,
        progress: TransferProgress,
    },
    /// A DCC session completed successfully.
    Complete {
        session_id: SessionId,
        peer_nick: String,
        /// Optional summary message (e.g., filename, bytes transferred).
        summary: String,
    },
    /// A DCC session failed with an error.
    Failed {
        session_id: SessionId,
        peer_nick: String,
        error: String,
    },
    /// A DCC session was cancelled.
    Cancelled {
        session_id: SessionId,
        peer_nick: String,
    },
    /// A chat message was received over DCC.
    ChatMessage {
        session_id: SessionId,
        peer_nick: String,
        message: String,
    },
}

/// Parsed DCC request extracted from a CTCP message.
#[derive(Debug, Clone)]
pub enum DccRequest {
    /// DCC CHAT request: `DCC CHAT chat <ip> <port>`
    Chat {
        peer_nick: String,
        address: IpAddr,
        port: u16,
    },
    /// DCC SEND request: `DCC SEND <filename> <ip> <port> <filesize>`
    Send {
        peer_nick: String,
        filename: String,
        address: IpAddr,
        port: u16,
        file_size: u64,
    },
    /// DCC RESUME request: `DCC RESUME <filename> <port> <position>`
    Resume {
        peer_nick: String,
        filename: String,
        port: u16,
        position: u64,
    },
    /// DCC ACCEPT response: `DCC ACCEPT <filename> <port> <position>`
    Accept {
        peer_nick: String,
        filename: String,
        port: u16,
        position: u64,
    },
}

/// Central manager for all DCC sessions.
///
/// Tracks active chats and file transfers, handles incoming DCC requests,
/// and emits [`DccEvent`]s through a channel for the application to consume.
pub struct DccManager {
    /// DCC configuration.
    config: DccConfig,
    /// Active sessions indexed by session ID.
    sessions: Arc<RwLock<HashMap<SessionId, DccSession>>>,
    /// Next session ID to assign.
    next_id: Arc<RwLock<SessionId>>,
    /// Channel sender for DCC events.
    event_tx: mpsc::UnboundedSender<DccEvent>,
    /// Channel receiver for DCC events (taken by the consumer).
    event_rx: Option<mpsc::UnboundedReceiver<DccEvent>>,
}

impl DccManager {
    /// Create a new DCC manager with the given configuration.
    pub fn new(config: DccConfig) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            event_tx,
            event_rx: Some(event_rx),
        }
    }

    /// Take the event receiver. Can only be called once; subsequent calls return `None`.
    pub fn take_event_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<DccEvent>> {
        self.event_rx.take()
    }

    /// Returns a clone of the event sender for use by sub-components.
    pub fn event_sender(&self) -> mpsc::UnboundedSender<DccEvent> {
        self.event_tx.clone()
    }

    /// Allocate the next unique session ID.
    async fn next_session_id(&self) -> SessionId {
        let mut id = self.next_id.write().await;
        let current = *id;
        *id += 1;
        current
    }

    /// Parse a raw CTCP DCC string into a [`DccRequest`].
    ///
    /// Expected formats:
    /// - `DCC CHAT chat <ip_long> <port>`
    /// - `DCC SEND <filename> <ip_long> <port> <filesize>`
    /// - `DCC RESUME <filename> <port> <position>`
    /// - `DCC ACCEPT <filename> <port> <position>`
    pub fn parse_dcc_request(peer_nick: &str, ctcp_data: &str) -> DccResult<DccRequest> {
        let parts: Vec<&str> = ctcp_data.split_whitespace().collect();
        if parts.len() < 4 {
            return Err(DccError::InvalidRequest(format!(
                "Too few parameters in DCC request: {ctcp_data}"
            )));
        }

        let dcc_type = parts[0].to_uppercase();
        match dcc_type.as_str() {
            "CHAT" => {
                // DCC CHAT chat <ip_long> <port>
                if parts.len() < 4 {
                    return Err(DccError::InvalidRequest(
                        "DCC CHAT requires: chat <ip> <port>".to_string(),
                    ));
                }
                let address = Self::parse_ip_long(parts[2])?;
                let port = parts[3]
                    .parse::<u16>()
                    .map_err(|_| DccError::InvalidRequest(format!("Invalid port: {}", parts[3])))?;
                Ok(DccRequest::Chat {
                    peer_nick: peer_nick.to_string(),
                    address,
                    port,
                })
            }
            "SEND" => {
                // DCC SEND <filename> <ip_long> <port> <filesize>
                if parts.len() < 5 {
                    return Err(DccError::InvalidRequest(
                        "DCC SEND requires: <filename> <ip> <port> <filesize>".to_string(),
                    ));
                }
                let filename = parts[1].to_string();
                let address = Self::parse_ip_long(parts[2])?;
                let port = parts[3]
                    .parse::<u16>()
                    .map_err(|_| DccError::InvalidRequest(format!("Invalid port: {}", parts[3])))?;
                let file_size = parts[4].parse::<u64>().map_err(|_| {
                    DccError::InvalidRequest(format!("Invalid file size: {}", parts[4]))
                })?;
                Ok(DccRequest::Send {
                    peer_nick: peer_nick.to_string(),
                    filename,
                    address,
                    port,
                    file_size,
                })
            }
            "RESUME" => {
                // DCC RESUME <filename> <port> <position>
                if parts.len() < 4 {
                    return Err(DccError::InvalidRequest(
                        "DCC RESUME requires: <filename> <port> <position>".to_string(),
                    ));
                }
                let filename = parts[1].to_string();
                let port = parts[2]
                    .parse::<u16>()
                    .map_err(|_| DccError::InvalidRequest(format!("Invalid port: {}", parts[2])))?;
                let position = parts[3].parse::<u64>().map_err(|_| {
                    DccError::InvalidRequest(format!("Invalid position: {}", parts[3]))
                })?;
                Ok(DccRequest::Resume {
                    peer_nick: peer_nick.to_string(),
                    filename,
                    port,
                    position,
                })
            }
            "ACCEPT" => {
                // DCC ACCEPT <filename> <port> <position>
                if parts.len() < 4 {
                    return Err(DccError::InvalidRequest(
                        "DCC ACCEPT requires: <filename> <port> <position>".to_string(),
                    ));
                }
                let filename = parts[1].to_string();
                let port = parts[2]
                    .parse::<u16>()
                    .map_err(|_| DccError::InvalidRequest(format!("Invalid port: {}", parts[2])))?;
                let position = parts[3].parse::<u64>().map_err(|_| {
                    DccError::InvalidRequest(format!("Invalid position: {}", parts[3]))
                })?;
                Ok(DccRequest::Accept {
                    peer_nick: peer_nick.to_string(),
                    filename,
                    port,
                    position,
                })
            }
            _ => Err(DccError::InvalidRequest(format!(
                "Unknown DCC type: {dcc_type}"
            ))),
        }
    }

    /// Parse a long-format IP address (32-bit integer) into an [`IpAddr`].
    ///
    /// DCC protocol encodes IPv4 addresses as a single unsigned 32-bit integer.
    /// For example, `127.0.0.1` is encoded as `2130706433`.
    fn parse_ip_long(s: &str) -> DccResult<IpAddr> {
        // Try parsing as a dotted-quad first (some modern clients use this).
        if let Ok(addr) = s.parse::<IpAddr>() {
            return Ok(addr);
        }
        // Parse as a long integer (traditional DCC encoding).
        let ip_long: u32 = s
            .parse()
            .map_err(|_| DccError::InvalidAddress(format!("Invalid IP address: {s}")))?;
        let octets = ip_long.to_be_bytes();
        Ok(IpAddr::V4(std::net::Ipv4Addr::new(
            octets[0], octets[1], octets[2], octets[3],
        )))
    }

    /// Convert an IP address to the long integer format used in DCC messages.
    pub fn ip_to_long(addr: &IpAddr) -> u64 {
        match addr {
            IpAddr::V4(v4) => {
                let octets = v4.octets();
                u64::from(u32::from_be_bytes(octets))
            }
            IpAddr::V6(_) => {
                // IPv6 is not supported in traditional DCC; use 0 as fallback.
                0
            }
        }
    }

    /// Handle an incoming DCC request parsed from CTCP.
    ///
    /// Creates a new session, registers it, and emits an [`DccEvent::Offered`] event.
    /// If `auto_accept` is enabled in the configuration, the session is automatically
    /// accepted.
    pub async fn handle_dcc_request(&self, request: DccRequest) -> DccResult<SessionId> {
        if !self.config.enabled {
            return Err(DccError::Disabled);
        }

        let session_id = self.next_session_id().await;

        let (session, description) = match &request {
            DccRequest::Chat {
                peer_nick,
                address,
                port,
                ..
            } => {
                let session = DccSession::Chat {
                    id: session_id,
                    peer_nick: peer_nick.clone(),
                    direction: DccDirection::Incoming,
                    remote_addr: Some(*address),
                    remote_port: Some(*port),
                    connected: false,
                };
                let desc = format!("DCC CHAT from {peer_nick} ({address}:{port})");
                (session, desc)
            }
            DccRequest::Send {
                peer_nick,
                filename,
                file_size,
                ..
            } => {
                // Check file size limit.
                if *file_size > self.config.max_file_size {
                    return Err(DccError::FileTooLarge {
                        size: *file_size,
                        limit: self.config.max_file_size,
                    });
                }
                let session = DccSession::Receive {
                    id: session_id,
                    peer_nick: peer_nick.clone(),
                    filename: filename.clone(),
                    file_size: *file_size,
                    progress: TransferProgress::new(*file_size),
                    active: false,
                };
                let desc = format!("DCC SEND {filename} from {peer_nick} ({} bytes)", file_size);
                (session, desc)
            }
            DccRequest::Resume { .. } | DccRequest::Accept { .. } => {
                // Resume and Accept are handled as protocol messages on existing sessions,
                // not as new session creation. Return the session_id as a no-op placeholder;
                // the actual resume logic lives in transfer.rs.
                return Ok(session_id);
            }
        };

        let peer_nick = session.peer_nick().to_string();

        self.sessions.write().await.insert(session_id, session);

        let _ = self.event_tx.send(DccEvent::Offered {
            session_id,
            peer_nick: peer_nick.clone(),
            description,
        });

        Ok(session_id)
    }

    /// Initiate an outgoing DCC CHAT session with the specified peer.
    ///
    /// Returns the session ID and the CTCP message string to send via IRC.
    pub async fn initiate_chat(
        &self,
        peer_nick: &str,
        local_addr: IpAddr,
        port: u16,
    ) -> DccResult<(SessionId, String)> {
        if !self.config.enabled {
            return Err(DccError::Disabled);
        }

        let session_id = self.next_session_id().await;
        let session = DccSession::Chat {
            id: session_id,
            peer_nick: peer_nick.to_string(),
            direction: DccDirection::Outgoing,
            remote_addr: None,
            remote_port: None,
            connected: false,
        };

        self.sessions.write().await.insert(session_id, session);

        let ip_long = Self::ip_to_long(&local_addr);
        let ctcp_msg = format!("\x01DCC CHAT chat {ip_long} {port}\x01");

        Ok((session_id, ctcp_msg))
    }

    /// Initiate an outgoing DCC SEND for a file.
    ///
    /// Returns the session ID and the CTCP message string to send via IRC.
    pub async fn initiate_send(
        &self,
        peer_nick: &str,
        filename: &str,
        file_size: u64,
        local_addr: IpAddr,
        port: u16,
    ) -> DccResult<(SessionId, String)> {
        if !self.config.enabled {
            return Err(DccError::Disabled);
        }

        if file_size > self.config.max_file_size {
            return Err(DccError::FileTooLarge {
                size: file_size,
                limit: self.config.max_file_size,
            });
        }

        let session_id = self.next_session_id().await;
        let session = DccSession::Send {
            id: session_id,
            peer_nick: peer_nick.to_string(),
            filename: filename.to_string(),
            file_size,
            progress: TransferProgress::new(file_size),
            active: false,
        };

        self.sessions.write().await.insert(session_id, session);

        let ip_long = Self::ip_to_long(&local_addr);
        // Sanitize filename: replace spaces with underscores for protocol compatibility.
        let safe_filename = filename.replace(' ', "_");
        let ctcp_msg = format!("\x01DCC SEND {safe_filename} {ip_long} {port} {file_size}\x01");

        Ok((session_id, ctcp_msg))
    }

    /// Accept a pending incoming DCC session (chat or transfer).
    ///
    /// Marks the session as active and emits an [`DccEvent::Accepted`] event.
    pub async fn accept_transfer(&self, session_id: SessionId) -> DccResult<DccSession> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or(DccError::SessionNotFound(session_id))?;

        match session {
            DccSession::Chat { connected, .. } => {
                *connected = true;
            }
            DccSession::Send { active, .. } | DccSession::Receive { active, .. } => {
                *active = true;
            }
        }

        let peer_nick = session.peer_nick().to_string();
        let result = session.clone();

        let _ = self.event_tx.send(DccEvent::Accepted {
            session_id,
            peer_nick,
        });

        Ok(result)
    }

    /// Cancel an active or pending DCC session.
    pub async fn cancel(&self, session_id: SessionId) -> DccResult<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .remove(&session_id)
            .ok_or(DccError::SessionNotFound(session_id))?;

        let _ = self.event_tx.send(DccEvent::Cancelled {
            session_id,
            peer_nick: session.peer_nick().to_string(),
        });

        Ok(())
    }

    /// List all active DCC sessions.
    pub async fn list_sessions(&self) -> Vec<DccSession> {
        self.sessions.read().await.values().cloned().collect()
    }

    /// Get a specific DCC session by ID.
    pub async fn get_session(&self, session_id: SessionId) -> DccResult<DccSession> {
        self.sessions
            .read()
            .await
            .get(&session_id)
            .cloned()
            .ok_or(DccError::SessionNotFound(session_id))
    }

    /// Update the progress of a transfer session.
    pub async fn update_progress(
        &self,
        session_id: SessionId,
        progress: TransferProgress,
    ) -> DccResult<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or(DccError::SessionNotFound(session_id))?;

        match session {
            DccSession::Send { progress: p, .. } | DccSession::Receive { progress: p, .. } => {
                *p = progress.clone();
            }
            DccSession::Chat { .. } => {
                // Chat sessions don't have transfer progress; ignore silently.
            }
        }

        let _ = self.event_tx.send(DccEvent::Progress {
            session_id,
            progress,
        });

        Ok(())
    }

    /// Mark a session as complete and remove it from active sessions.
    pub async fn complete_session(&self, session_id: SessionId, summary: String) -> DccResult<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .remove(&session_id)
            .ok_or(DccError::SessionNotFound(session_id))?;

        let _ = self.event_tx.send(DccEvent::Complete {
            session_id,
            peer_nick: session.peer_nick().to_string(),
            summary,
        });

        Ok(())
    }

    /// Mark a session as failed and remove it from active sessions.
    pub async fn fail_session(&self, session_id: SessionId, error: String) -> DccResult<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .remove(&session_id)
            .ok_or(DccError::SessionNotFound(session_id))?;

        let _ = self.event_tx.send(DccEvent::Failed {
            session_id,
            peer_nick: session.peer_nick().to_string(),
            error,
        });

        Ok(())
    }

    /// Returns a reference to the DCC configuration.
    pub fn config(&self) -> &DccConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_parse_dcc_chat_request() {
        let req = DccManager::parse_dcc_request("alice", "CHAT chat 2130706433 12345").unwrap();
        match req {
            DccRequest::Chat {
                peer_nick,
                address,
                port,
            } => {
                assert_eq!(peer_nick, "alice");
                assert_eq!(address, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
                assert_eq!(port, 12345);
            }
            _ => panic!("Expected DCC CHAT request"),
        }
    }

    #[test]
    fn test_parse_dcc_send_request() {
        let req = DccManager::parse_dcc_request("bob", "SEND myfile.txt 3232235521 54321 1048576")
            .unwrap();
        match req {
            DccRequest::Send {
                peer_nick,
                filename,
                address,
                port,
                file_size,
            } => {
                assert_eq!(peer_nick, "bob");
                assert_eq!(filename, "myfile.txt");
                assert_eq!(address, IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)));
                assert_eq!(port, 54321);
                assert_eq!(file_size, 1048576);
            }
            _ => panic!("Expected DCC SEND request"),
        }
    }

    #[test]
    fn test_parse_dcc_resume_request() {
        let req =
            DccManager::parse_dcc_request("carol", "RESUME largefile.zip 54321 524288").unwrap();
        match req {
            DccRequest::Resume {
                peer_nick,
                filename,
                port,
                position,
            } => {
                assert_eq!(peer_nick, "carol");
                assert_eq!(filename, "largefile.zip");
                assert_eq!(port, 54321);
                assert_eq!(position, 524288);
            }
            _ => panic!("Expected DCC RESUME request"),
        }
    }

    #[test]
    fn test_parse_dcc_accept_request() {
        let req =
            DccManager::parse_dcc_request("dave", "ACCEPT largefile.zip 54321 524288").unwrap();
        match req {
            DccRequest::Accept {
                peer_nick,
                filename,
                port,
                position,
            } => {
                assert_eq!(peer_nick, "dave");
                assert_eq!(filename, "largefile.zip");
                assert_eq!(port, 54321);
                assert_eq!(position, 524288);
            }
            _ => panic!("Expected DCC ACCEPT request"),
        }
    }

    #[test]
    fn test_parse_invalid_request() {
        let result = DccManager::parse_dcc_request("nick", "INVALID");
        assert!(result.is_err());

        let result = DccManager::parse_dcc_request("nick", "SEND");
        assert!(result.is_err());
    }

    #[test]
    fn test_ip_long_conversion() {
        let addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        assert_eq!(DccManager::ip_to_long(&addr), 2130706433);

        let addr = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1));
        assert_eq!(DccManager::ip_to_long(&addr), 3232235521);
    }

    #[test]
    fn test_ip_long_roundtrip() {
        let original = IpAddr::V4(Ipv4Addr::new(10, 20, 30, 40));
        let long_val = DccManager::ip_to_long(&original);
        let parsed = DccManager::parse_ip_long(&long_val.to_string()).unwrap();
        assert_eq!(original, parsed);
    }

    #[tokio::test]
    async fn test_session_creation_and_lookup() {
        let config = DccConfig::default();
        let manager = DccManager::new(config);

        let request = DccRequest::Chat {
            peer_nick: "alice".to_string(),
            address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 12345,
        };

        let session_id = manager.handle_dcc_request(request).await.unwrap();
        let session = manager.get_session(session_id).await.unwrap();
        assert_eq!(session.peer_nick(), "alice");
        assert!(!session.is_active());
    }

    #[tokio::test]
    async fn test_accept_transfer() {
        let config = DccConfig::default();
        let manager = DccManager::new(config);

        let request = DccRequest::Send {
            peer_nick: "bob".to_string(),
            filename: "test.txt".to_string(),
            address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 54321,
            file_size: 1024,
        };

        let session_id = manager.handle_dcc_request(request).await.unwrap();
        let session = manager.accept_transfer(session_id).await.unwrap();
        assert!(session.is_active());
    }

    #[tokio::test]
    async fn test_cancel_session() {
        let config = DccConfig::default();
        let manager = DccManager::new(config);

        let request = DccRequest::Chat {
            peer_nick: "eve".to_string(),
            address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 9999,
        };

        let session_id = manager.handle_dcc_request(request).await.unwrap();
        manager.cancel(session_id).await.unwrap();

        let result = manager.get_session(session_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let config = DccConfig::default();
        let manager = DccManager::new(config);

        let req1 = DccRequest::Chat {
            peer_nick: "alice".to_string(),
            address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 1111,
        };
        let req2 = DccRequest::Chat {
            peer_nick: "bob".to_string(),
            address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 2222,
        };

        manager.handle_dcc_request(req1).await.unwrap();
        manager.handle_dcc_request(req2).await.unwrap();

        let sessions = manager.list_sessions().await;
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_dcc_disabled() {
        let config = DccConfig {
            enabled: false,
            ..DccConfig::default()
        };
        let manager = DccManager::new(config);

        let request = DccRequest::Chat {
            peer_nick: "alice".to_string(),
            address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 1234,
        };

        let result = manager.handle_dcc_request(request).await;
        assert!(matches!(result, Err(DccError::Disabled)));
    }

    #[tokio::test]
    async fn test_file_too_large() {
        let config = DccConfig {
            max_file_size: 1024,
            ..DccConfig::default()
        };
        let manager = DccManager::new(config);

        let request = DccRequest::Send {
            peer_nick: "bob".to_string(),
            filename: "huge.bin".to_string(),
            address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 5555,
            file_size: 2048,
        };

        let result = manager.handle_dcc_request(request).await;
        assert!(matches!(result, Err(DccError::FileTooLarge { .. })));
    }

    #[tokio::test]
    async fn test_initiate_chat() {
        let config = DccConfig::default();
        let manager = DccManager::new(config);

        let (session_id, ctcp_msg) = manager
            .initiate_chat("alice", IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 12345)
            .await
            .unwrap();

        assert!(ctcp_msg.contains("DCC CHAT chat"));
        assert!(ctcp_msg.contains("12345"));

        let session = manager.get_session(session_id).await.unwrap();
        assert_eq!(session.peer_nick(), "alice");
    }

    #[tokio::test]
    async fn test_initiate_send() {
        let config = DccConfig::default();
        let manager = DccManager::new(config);

        let (session_id, ctcp_msg) = manager
            .initiate_send(
                "bob",
                "my file.txt",
                4096,
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
                54321,
            )
            .await
            .unwrap();

        // Spaces should be replaced with underscores in the CTCP message.
        assert!(ctcp_msg.contains("my_file.txt"));
        assert!(ctcp_msg.contains("4096"));

        let session = manager.get_session(session_id).await.unwrap();
        assert_eq!(session.peer_nick(), "bob");
    }

    #[tokio::test]
    async fn test_complete_session() {
        let config = DccConfig::default();
        let mut manager = DccManager::new(config);
        let mut rx = manager.take_event_receiver().unwrap();

        let request = DccRequest::Chat {
            peer_nick: "alice".to_string(),
            address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 1234,
        };

        let session_id = manager.handle_dcc_request(request).await.unwrap();
        // Drain the Offered event.
        let _ = rx.recv().await;

        manager
            .complete_session(session_id, "Chat ended".to_string())
            .await
            .unwrap();

        // Session should be removed.
        assert!(manager.get_session(session_id).await.is_err());

        // Should have received a Complete event.
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, DccEvent::Complete { .. }));
    }

    #[tokio::test]
    async fn test_session_not_found() {
        let config = DccConfig::default();
        let manager = DccManager::new(config);

        let result = manager.get_session(999).await;
        assert!(matches!(result, Err(DccError::SessionNotFound(999))));

        let result = manager.cancel(999).await;
        assert!(matches!(result, Err(DccError::SessionNotFound(999))));
    }

    #[test]
    fn test_session_display() {
        let session = DccSession::Send {
            id: 1,
            peer_nick: "alice".to_string(),
            filename: "test.txt".to_string(),
            file_size: 1024,
            progress: TransferProgress {
                bytes_transferred: 512,
                total_bytes: 1024,
                speed_bps: 100.0,
                percentage: 50.0,
            },
            active: true,
        };
        let display = format!("{session}");
        assert!(display.contains("DCC SEND"));
        assert!(display.contains("test.txt"));
        assert!(display.contains("alice"));
        assert!(display.contains("50.0%"));
    }

    #[test]
    fn test_dcc_direction_display() {
        assert_eq!(format!("{}", DccDirection::Outgoing), "outgoing");
        assert_eq!(format!("{}", DccDirection::Incoming), "incoming");
    }

    #[test]
    fn test_parse_dotted_quad_ip() {
        let addr = DccManager::parse_ip_long("192.168.1.1").unwrap();
        assert_eq!(addr, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
    }
}
