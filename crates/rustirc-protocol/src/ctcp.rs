//! CTCP (Client-To-Client Protocol) support
//!
//! Implements CTCP message handling including ACTION, VERSION, TIME, and other
//! client-to-client protocol commands as specified in the CTCP specification.

use crate::{Message, Prefix};
use std::time::{SystemTime, UNIX_EPOCH};

/// CTCP message types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CtcpMessage {
    /// ACTION message (/me command)
    Action { text: String },
    /// VERSION request/response
    Version { response: Option<String> },
    /// TIME request/response
    Time { timestamp: Option<String> },
    /// PING request/response
    Ping { token: String },
    /// FINGER request/response
    Finger { response: Option<String> },
    /// USERINFO request/response
    UserInfo { response: Option<String> },
    /// CLIENTINFO request/response
    ClientInfo { response: Option<String> },
    /// SOURCE request/response
    Source { response: Option<String> },
    /// Custom CTCP command
    Custom {
        command: String,
        data: Option<String>,
    },
}

/// CTCP parser and handler
pub struct CtcpHandler {
    /// Client version string
    pub version: String,
    /// Client name
    pub client_name: String,
    /// User information
    pub user_info: Option<String>,
    /// Finger information
    pub finger_info: Option<String>,
    /// Source information
    pub source_info: Option<String>,
}

impl CtcpHandler {
    /// Create a new CTCP handler
    pub fn new(client_name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            client_name: client_name.into(),
            user_info: None,
            finger_info: None,
            source_info: None,
        }
    }

    /// Check if a message is a CTCP message
    pub fn is_ctcp_message(message: &Message) -> bool {
        if message.command != "PRIVMSG" && message.command != "NOTICE" {
            return false;
        }

        if message.params.len() < 2 {
            return false;
        }

        let text = &message.params[1];
        text.starts_with('\x01') && text.ends_with('\x01') && text.len() > 2
    }

    /// Parse a CTCP message from an IRC message
    pub fn parse_ctcp_message(message: &Message) -> Option<CtcpMessage> {
        if !Self::is_ctcp_message(message) {
            return None;
        }

        let text = &message.params[1];
        // Remove CTCP delimiters
        let ctcp_content = &text[1..text.len() - 1];

        let parts: Vec<&str> = ctcp_content.splitn(2, ' ').collect();
        let command = parts[0].to_uppercase();
        let data = parts.get(1).map(|s| s.to_string());

        match command.as_str() {
            "ACTION" => Some(CtcpMessage::Action {
                text: data.unwrap_or_default(),
            }),
            "VERSION" => Some(CtcpMessage::Version { response: data }),
            "TIME" => Some(CtcpMessage::Time { timestamp: data }),
            "PING" => Some(CtcpMessage::Ping {
                token: data.unwrap_or_default(),
            }),
            "FINGER" => Some(CtcpMessage::Finger { response: data }),
            "USERINFO" => Some(CtcpMessage::UserInfo { response: data }),
            "CLIENTINFO" => Some(CtcpMessage::ClientInfo { response: data }),
            "SOURCE" => Some(CtcpMessage::Source { response: data }),
            _ => Some(CtcpMessage::Custom {
                command: command.to_lowercase(),
                data,
            }),
        }
    }

    /// Create a CTCP ACTION message
    pub fn create_action(target: impl Into<String>, text: impl Into<String>) -> Message {
        let ctcp_text = format!("\x01ACTION {}\x01", text.into());
        Message::new("PRIVMSG")
            .add_param(target.into())
            .add_param(ctcp_text)
    }

    /// Create a CTCP VERSION request
    pub fn create_version_request(target: impl Into<String>) -> Message {
        Message::new("PRIVMSG")
            .add_param(target.into())
            .add_param("\x01VERSION\x01")
    }

    /// Create a CTCP TIME request
    pub fn create_time_request(target: impl Into<String>) -> Message {
        Message::new("PRIVMSG")
            .add_param(target.into())
            .add_param("\x01TIME\x01")
    }

    /// Create a CTCP PING request
    pub fn create_ping_request(target: impl Into<String>, token: impl Into<String>) -> Message {
        let ctcp_text = format!("\x01PING {}\x01", token.into());
        Message::new("PRIVMSG")
            .add_param(target.into())
            .add_param(ctcp_text)
    }

    /// Handle a CTCP request and generate appropriate response
    pub fn handle_ctcp_request(&self, message: &Message) -> Option<Message> {
        if message.command != "PRIVMSG" {
            return None;
        }

        let ctcp_msg = Self::parse_ctcp_message(message)?;
        let sender = match &message.prefix {
            Some(Prefix::User { nick, .. }) => nick,
            Some(Prefix::Server(server)) => server,
            None => return None,
        };

        match ctcp_msg {
            CtcpMessage::Version { .. } => {
                let response = format!("\x01VERSION {}\x01", self.version);
                Some(Message::new("NOTICE").add_param(sender).add_param(response))
            }
            CtcpMessage::Time { .. } => {
                let timestamp = self.get_current_time();
                let response = format!("\x01TIME {timestamp}\x01");
                Some(Message::new("NOTICE").add_param(sender).add_param(response))
            }
            CtcpMessage::Ping { token } => {
                let response = format!("\x01PING {token}\x01");
                Some(Message::new("NOTICE").add_param(sender).add_param(response))
            }
            CtcpMessage::Finger { .. } => {
                if let Some(finger_info) = &self.finger_info {
                    let response = format!("\x01FINGER {finger_info}\x01");
                    Some(Message::new("NOTICE").add_param(sender).add_param(response))
                } else {
                    None
                }
            }
            CtcpMessage::UserInfo { .. } => {
                if let Some(user_info) = &self.user_info {
                    let response = format!("\x01USERINFO {user_info}\x01");
                    Some(Message::new("NOTICE").add_param(sender).add_param(response))
                } else {
                    None
                }
            }
            CtcpMessage::ClientInfo { .. } => {
                let supported_commands =
                    "ACTION VERSION TIME PING FINGER USERINFO CLIENTINFO SOURCE";
                let response = format!("\x01CLIENTINFO {supported_commands}\x01");
                Some(Message::new("NOTICE").add_param(sender).add_param(response))
            }
            CtcpMessage::Source { .. } => {
                if let Some(source_info) = &self.source_info {
                    let response = format!("\x01SOURCE {source_info}\x01");
                    Some(Message::new("NOTICE").add_param(sender).add_param(response))
                } else {
                    None
                }
            }
            // Don't auto-respond to ACTION or custom messages
            _ => None,
        }
    }

    /// Get current time as RFC 2822 formatted string
    fn get_current_time(&self) -> String {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                // Simple timestamp format - could be enhanced with proper RFC 2822 formatting
                format!("{}", duration.as_secs())
            }
            Err(_) => "Unknown".to_string(),
        }
    }

    /// Set user information for USERINFO responses
    pub fn set_user_info(&mut self, info: impl Into<String>) {
        self.user_info = Some(info.into());
    }

    /// Set finger information for FINGER responses
    pub fn set_finger_info(&mut self, info: impl Into<String>) {
        self.finger_info = Some(info.into());
    }

    /// Set source information for SOURCE responses
    pub fn set_source_info(&mut self, info: impl Into<String>) {
        self.source_info = Some(info.into());
    }

    /// Extract ACTION text from a CTCP ACTION message
    pub fn extract_action_text(message: &Message) -> Option<String> {
        if let Some(CtcpMessage::Action { text }) = Self::parse_ctcp_message(message) {
            Some(text)
        } else {
            None
        }
    }

    /// Check if message is a CTCP ACTION
    pub fn is_action(message: &Message) -> bool {
        matches!(
            Self::parse_ctcp_message(message),
            Some(CtcpMessage::Action { .. })
        )
    }
}

/// Escape CTCP message content (low-level quoting)
pub fn escape_ctcp(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('\x01', "\\a")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\0', "\\0")
}

/// Unescape CTCP message content (low-level unquoting)  
pub fn unescape_ctcp(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('\\') => result.push('\\'),
                Some('a') => result.push('\x01'),
                Some('r') => result.push('\r'),
                Some('n') => result.push('\n'),
                Some('0') => result.push('\0'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctcp_detection() {
        let action_msg = Message::new("PRIVMSG")
            .add_param("#channel")
            .add_param("\x01ACTION waves\x01");
        assert!(CtcpHandler::is_ctcp_message(&action_msg));

        let normal_msg = Message::new("PRIVMSG")
            .add_param("#channel")
            .add_param("Hello everyone");
        assert!(!CtcpHandler::is_ctcp_message(&normal_msg));
    }

    #[test]
    fn test_action_parsing() {
        let action_msg = Message::new("PRIVMSG")
            .add_param("#channel")
            .add_param("\x01ACTION waves at everyone\x01");

        let parsed = CtcpHandler::parse_ctcp_message(&action_msg);
        assert_eq!(
            parsed,
            Some(CtcpMessage::Action {
                text: "waves at everyone".to_string()
            })
        );
    }

    #[test]
    fn test_version_request() {
        let version_msg = Message::new("PRIVMSG")
            .add_param("nick")
            .add_param("\x01VERSION\x01");

        let parsed = CtcpHandler::parse_ctcp_message(&version_msg);
        assert_eq!(parsed, Some(CtcpMessage::Version { response: None }));
    }

    #[test]
    fn test_version_response() {
        let handler = CtcpHandler::new("RustIRC", "1.0.0");
        let request = Message::new("PRIVMSG")
            .with_prefix(Prefix::User {
                nick: "requester".to_string(),
                user: Some("user".to_string()),
                host: Some("host.example.com".to_string()),
            })
            .add_param("mynick")
            .add_param("\x01VERSION\x01");

        let response = handler.handle_ctcp_request(&request);
        assert!(response.is_some());

        let response = response.unwrap();
        assert_eq!(response.command, "NOTICE");
        assert_eq!(response.params[0], "requester");
        assert!(response.params[1].contains("1.0.0"));
    }

    #[test]
    fn test_ping_round_trip() {
        let handler = CtcpHandler::new("RustIRC", "1.0.0");
        // Generate a test PING token for CTCP testing (not a security credential)
        let ping_token = format!(
            "test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                % 1000
        );

        let request = Message::new("PRIVMSG")
            .with_prefix(Prefix::User {
                nick: "pinger".to_string(),
                user: None,
                host: None,
            })
            .add_param("mynick")
            .add_param(format!("\x01PING {}\x01", ping_token));

        let response = handler.handle_ctcp_request(&request);
        assert!(response.is_some());

        let response = response.unwrap();
        assert_eq!(response.command, "NOTICE");
        assert_eq!(response.params[0], "pinger");
        assert!(response.params[1].contains(ping_token));
    }

    #[test]
    fn test_action_creation() {
        let action = CtcpHandler::create_action("#channel", "waves at everyone");
        assert_eq!(action.command, "PRIVMSG");
        assert_eq!(action.params[0], "#channel");
        assert_eq!(action.params[1], "\x01ACTION waves at everyone\x01");
    }

    #[test]
    fn test_ctcp_escaping() {
        assert_eq!(escape_ctcp("hello\\world"), "hello\\\\world");
        assert_eq!(escape_ctcp("hello\x01world"), "hello\\aworld");
        assert_eq!(escape_ctcp("hello\r\nworld"), "hello\\r\\nworld");

        assert_eq!(unescape_ctcp("hello\\\\world"), "hello\\world");
        assert_eq!(unescape_ctcp("hello\\aworld"), "hello\x01world");
        assert_eq!(unescape_ctcp("hello\\r\\nworld"), "hello\r\nworld");
    }

    #[test]
    fn test_extract_action_text() {
        let action_msg = Message::new("PRIVMSG")
            .add_param("#channel")
            .add_param("\x01ACTION is happy\x01");

        let text = CtcpHandler::extract_action_text(&action_msg);
        assert_eq!(text, Some("is happy".to_string()));
    }

    #[test]
    fn test_is_action() {
        let action_msg = Message::new("PRIVMSG")
            .add_param("#channel")
            .add_param("\x01ACTION is excited\x01");
        assert!(CtcpHandler::is_action(&action_msg));

        let normal_msg = Message::new("PRIVMSG")
            .add_param("#channel")
            .add_param("Hello");
        assert!(!CtcpHandler::is_action(&normal_msg));
    }
}
