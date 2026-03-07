//! DCC CHAT implementation
//!
//! Provides direct peer-to-peer chat connections between IRC clients.
//! DCC CHAT bypasses the IRC server entirely, establishing a direct TCP
//! connection for private messaging.
//!
//! # Protocol Flow
//!
//! **Initiator (outgoing):**
//! 1. Bind a TCP listener on a local port.
//! 2. Send a CTCP `DCC CHAT chat <ip_long> <port>` to the target nick.
//! 3. Wait for the remote peer to connect.
//! 4. Exchange line-delimited text over the TCP stream.
//!
//! **Acceptor (incoming):**
//! 1. Receive the CTCP `DCC CHAT` request.
//! 2. Connect to the offered `address:port`.
//! 3. Exchange line-delimited text over the TCP stream.

use super::{DccError, DccEvent, DccManager, DccResult, SessionId};
use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

/// A DCC CHAT session managing a direct TCP connection for text messaging.
pub struct DccChat {
    /// The session ID in the DCC manager.
    session_id: SessionId,
    /// The remote peer's IRC nickname.
    peer_nick: String,
    /// The underlying TCP stream, wrapped in a buffered reader for line-oriented I/O.
    reader: Option<BufReader<tokio::io::ReadHalf<TcpStream>>>,
    /// The write half of the TCP stream.
    writer: Option<tokio::io::WriteHalf<TcpStream>>,
    /// Event sender for notifying the application of chat events.
    event_tx: mpsc::UnboundedSender<DccEvent>,
}

impl DccChat {
    /// Create a new `DccChat` instance (before the TCP connection is established).
    pub fn new(
        session_id: SessionId,
        peer_nick: String,
        event_tx: mpsc::UnboundedSender<DccEvent>,
    ) -> Self {
        Self {
            session_id,
            peer_nick,
            reader: None,
            writer: None,
            event_tx,
        }
    }

    /// Initiate a DCC CHAT by listening on a local port and waiting for the peer to connect.
    ///
    /// Returns the CTCP message string that should be sent to the peer via IRC PRIVMSG.
    /// The caller should send this CTCP message and then call [`wait_for_connection`] to
    /// accept the incoming TCP connection.
    ///
    /// # Arguments
    ///
    /// * `local_addr` - The local IP address to bind to.
    /// * `port` - The port to listen on. If `0`, the OS will assign an available port.
    ///
    /// # Returns
    ///
    /// A tuple of `(TcpListener, ctcp_message_string)`. The listener should be passed
    /// to [`wait_for_connection`].
    pub async fn initiate(local_addr: IpAddr, port: u16) -> DccResult<(TcpListener, u16, String)> {
        let bind_addr = SocketAddr::new(local_addr, port);
        let listener = TcpListener::bind(bind_addr)
            .await
            .map_err(|e| DccError::ConnectionFailed(format!("Failed to bind {bind_addr}: {e}")))?;

        let actual_port = listener.local_addr().map_err(DccError::Io)?.port();

        let ip_long = DccManager::ip_to_long(&local_addr);
        let ctcp_msg = format!("\x01DCC CHAT chat {ip_long} {actual_port}\x01");

        Ok((listener, actual_port, ctcp_msg))
    }

    /// Wait for a peer to connect on the given listener and establish the chat session.
    ///
    /// This should be called after sending the CTCP DCC CHAT message to the peer.
    pub async fn wait_for_connection(&mut self, listener: TcpListener) -> DccResult<()> {
        let (stream, _remote_addr) = listener
            .accept()
            .await
            .map_err(|e| DccError::ConnectionFailed(format!("Failed to accept connection: {e}")))?;

        let (read_half, write_half) = tokio::io::split(stream);
        self.reader = Some(BufReader::new(read_half));
        self.writer = Some(write_half);

        let _ = self.event_tx.send(DccEvent::Accepted {
            session_id: self.session_id,
            peer_nick: self.peer_nick.clone(),
        });

        Ok(())
    }

    /// Accept an incoming DCC CHAT by connecting to the peer's offered address and port.
    pub async fn accept(&mut self, address: IpAddr, port: u16) -> DccResult<()> {
        let remote_addr = SocketAddr::new(address, port);
        let stream = TcpStream::connect(remote_addr).await.map_err(|e| {
            DccError::ConnectionFailed(format!("Failed to connect to {remote_addr}: {e}"))
        })?;

        let (read_half, write_half) = tokio::io::split(stream);
        self.reader = Some(BufReader::new(read_half));
        self.writer = Some(write_half);

        let _ = self.event_tx.send(DccEvent::Accepted {
            session_id: self.session_id,
            peer_nick: self.peer_nick.clone(),
        });

        Ok(())
    }

    /// Send a line of text to the peer.
    ///
    /// Appends `\n` to the message before sending, as per the DCC CHAT protocol.
    pub async fn send_line(&mut self, line: &str) -> DccResult<()> {
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| DccError::ConnectionFailed("Chat not connected".to_string()))?;

        let data = format!("{line}\n");
        writer
            .write_all(data.as_bytes())
            .await
            .map_err(DccError::Io)?;
        writer.flush().await.map_err(DccError::Io)?;

        Ok(())
    }

    /// Receive a line of text from the peer.
    ///
    /// Returns `None` if the connection has been closed by the peer.
    pub async fn recv_line(&mut self) -> DccResult<Option<String>> {
        let reader = self
            .reader
            .as_mut()
            .ok_or_else(|| DccError::ConnectionFailed("Chat not connected".to_string()))?;

        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).await.map_err(DccError::Io)?;

        if bytes_read == 0 {
            // Connection closed by peer.
            return Ok(None);
        }

        // Strip trailing newline characters.
        let trimmed = line.trim_end_matches(['\n', '\r']).to_string();

        let _ = self.event_tx.send(DccEvent::ChatMessage {
            session_id: self.session_id,
            peer_nick: self.peer_nick.clone(),
            message: trimmed.clone(),
        });

        Ok(Some(trimmed))
    }

    /// Close the DCC CHAT connection.
    pub async fn close(&mut self) -> DccResult<()> {
        if let Some(mut writer) = self.writer.take() {
            let _ = writer.shutdown().await;
        }
        self.reader = None;

        let _ = self.event_tx.send(DccEvent::Complete {
            session_id: self.session_id,
            peer_nick: self.peer_nick.clone(),
            summary: "DCC CHAT closed".to_string(),
        });

        Ok(())
    }

    /// Returns the session ID.
    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    /// Returns the peer nickname.
    pub fn peer_nick(&self) -> &str {
        &self.peer_nick
    }

    /// Returns whether the chat is currently connected.
    pub fn is_connected(&self) -> bool {
        self.writer.is_some() && self.reader.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// Helper to create a chat with a dummy event channel.
    fn make_chat(
        session_id: SessionId,
        nick: &str,
    ) -> (DccChat, mpsc::UnboundedReceiver<DccEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let chat = DccChat::new(session_id, nick.to_string(), tx);
        (chat, rx)
    }

    #[tokio::test]
    async fn test_chat_initiate_and_accept_lifecycle() {
        // Initiator side: bind a listener.
        let (listener, port, ctcp_msg) = DccChat::initiate(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
            .await
            .unwrap();

        assert!(port > 0);
        assert!(ctcp_msg.contains("DCC CHAT chat"));

        let (mut initiator_chat, _rx1) = make_chat(1, "acceptor");
        let (mut acceptor_chat, _rx2) = make_chat(2, "initiator");

        // Spawn the initiator waiting for connection.
        let initiator_handle = tokio::spawn(async move {
            initiator_chat.wait_for_connection(listener).await.unwrap();
            initiator_chat
        });

        // Acceptor connects.
        acceptor_chat
            .accept(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
            .await
            .unwrap();

        let mut initiator_chat = initiator_handle.await.unwrap();

        // Both sides should be connected.
        assert!(initiator_chat.is_connected());
        assert!(acceptor_chat.is_connected());

        // Exchange messages.
        acceptor_chat
            .send_line("Hello from acceptor")
            .await
            .unwrap();
        let received = initiator_chat.recv_line().await.unwrap();
        assert_eq!(received, Some("Hello from acceptor".to_string()));

        initiator_chat
            .send_line("Hello from initiator")
            .await
            .unwrap();
        let received = acceptor_chat.recv_line().await.unwrap();
        assert_eq!(received, Some("Hello from initiator".to_string()));

        // Close both sides.
        initiator_chat.close().await.unwrap();
        acceptor_chat.close().await.unwrap();

        assert!(!initiator_chat.is_connected());
        assert!(!acceptor_chat.is_connected());
    }

    #[tokio::test]
    async fn test_chat_peer_disconnect_detection() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let (mut chat, _rx) = make_chat(1, "peer");

        // Spawn a connection that writes one line and then drops.
        let handle = tokio::spawn(async move {
            let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))
                .await
                .unwrap();
            stream.write_all(b"goodbye\n").await.unwrap();
            stream.shutdown().await.unwrap();
            // Stream dropped here, closing the connection.
        });

        chat.wait_for_connection(listener).await.unwrap();

        // Should read the line.
        let msg = chat.recv_line().await.unwrap();
        assert_eq!(msg, Some("goodbye".to_string()));

        // Next read should detect closure.
        let msg = chat.recv_line().await.unwrap();
        assert_eq!(msg, None);

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_send_line_not_connected() {
        let (mut chat, _rx) = make_chat(1, "nobody");
        let result = chat.send_line("test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_recv_line_not_connected() {
        let (mut chat, _rx) = make_chat(1, "nobody");
        let result = chat.recv_line().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_chat_events_emitted() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut chat = DccChat::new(1, "alice".to_string(), tx);

        // Connect from another task.
        let connect_handle = tokio::spawn(async move {
            let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))
                .await
                .unwrap();
            stream.write_all(b"hi there\n").await.unwrap();
            // Keep stream alive until we read.
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            stream.shutdown().await.unwrap();
        });

        chat.wait_for_connection(listener).await.unwrap();

        // Should get Accepted event.
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, DccEvent::Accepted { session_id: 1, .. }));

        // Read a message -- should get ChatMessage event.
        let _ = chat.recv_line().await.unwrap();
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, DccEvent::ChatMessage { session_id: 1, .. }));

        // Close should emit Complete.
        chat.close().await.unwrap();
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, DccEvent::Complete { session_id: 1, .. }));

        connect_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_multiline_exchange() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let (mut server_chat, _rx1) = make_chat(1, "client");
        let (mut client_chat, _rx2) = make_chat(2, "server");

        let server_handle = tokio::spawn(async move {
            server_chat.wait_for_connection(listener).await.unwrap();
            server_chat
        });

        client_chat
            .accept(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
            .await
            .unwrap();

        let mut server_chat = server_handle.await.unwrap();

        // Send multiple lines from client.
        let messages = vec!["line one", "line two", "line three"];
        for msg in &messages {
            client_chat.send_line(msg).await.unwrap();
        }

        // Server should receive all lines in order.
        for expected in &messages {
            let received = server_chat.recv_line().await.unwrap().unwrap();
            assert_eq!(received, *expected);
        }

        client_chat.close().await.unwrap();
        server_chat.close().await.unwrap();
    }
}
