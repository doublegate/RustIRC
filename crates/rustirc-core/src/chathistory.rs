//! IRCv3 CHATHISTORY support
//!
//! Implements the IRCv3 `draft/chathistory` specification for requesting
//! historical messages from an IRC server.
//!
//! See: <https://ircv3.net/specs/extensions/chathistory>

use std::collections::VecDeque;

use rustirc_protocol::Message;

/// Represents the different types of CHATHISTORY requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryRequest {
    /// Request messages before a given timestamp or msgid.
    /// `CHATHISTORY BEFORE <target> <reference> <limit>`
    Before {
        target: String,
        reference: MessageReference,
        limit: usize,
    },
    /// Request messages after a given timestamp or msgid.
    /// `CHATHISTORY AFTER <target> <reference> <limit>`
    After {
        target: String,
        reference: MessageReference,
        limit: usize,
    },
    /// Request messages between two timestamps or msgids.
    /// `CHATHISTORY BETWEEN <target> <start_ref> <end_ref> <limit>`
    Between {
        target: String,
        start_ref: MessageReference,
        end_ref: MessageReference,
        limit: usize,
    },
    /// Request messages around a given timestamp or msgid.
    /// `CHATHISTORY AROUND <target> <reference> <limit>`
    Around {
        target: String,
        reference: MessageReference,
        limit: usize,
    },
    /// Request the latest messages for a target.
    /// `CHATHISTORY LATEST <target> <reference_or_*> <limit>`
    Latest {
        target: String,
        reference: Option<MessageReference>,
        limit: usize,
    },
}

/// A reference point for CHATHISTORY requests - either a message ID or a
/// server-time timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageReference {
    /// Reference by message ID (`msgid=<id>`).
    MsgId(String),
    /// Reference by server time (`timestamp=<iso8601>`).
    Timestamp(String),
}

impl MessageReference {
    /// Format the reference for use in a CHATHISTORY command parameter.
    pub fn to_param(&self) -> String {
        match self {
            MessageReference::MsgId(id) => format!("msgid={id}"),
            MessageReference::Timestamp(ts) => format!("timestamp={ts}"),
        }
    }

    /// Parse a CHATHISTORY reference parameter.
    pub fn parse(param: &str) -> Option<Self> {
        if let Some(id) = param.strip_prefix("msgid=") {
            Some(MessageReference::MsgId(id.to_string()))
        } else {
            param
                .strip_prefix("timestamp=")
                .map(|ts| MessageReference::Timestamp(ts.to_string()))
        }
    }
}

/// A pending history request paired with a unique identifier for correlation.
#[derive(Debug, Clone)]
struct PendingRequest {
    /// Unique ID for tracking this request (could be a label tag value).
    id: u64,
    /// The actual request details.
    request: HistoryRequest,
}

/// Manages CHATHISTORY requests and response correlation.
///
/// Maintains a queue of pending requests, builds the protocol messages,
/// and provides helpers for processing server responses.
#[derive(Debug)]
pub struct ChatHistoryManager {
    /// Queue of pending requests awaiting server responses.
    pending: VecDeque<PendingRequest>,
    /// Next request ID for correlation.
    next_id: u64,
    /// Maximum number of messages to request per query if not specified.
    pub default_limit: usize,
}

impl Default for ChatHistoryManager {
    fn default() -> Self {
        Self {
            pending: VecDeque::new(),
            next_id: 1,
            default_limit: 100,
        }
    }
}

impl ChatHistoryManager {
    /// Create a new `ChatHistoryManager`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new `ChatHistoryManager` with a custom default limit.
    pub fn with_default_limit(limit: usize) -> Self {
        Self {
            default_limit: limit,
            ..Self::default()
        }
    }

    /// Queue a history request and return the protocol `Message` to send,
    /// along with a request ID for correlation.
    pub fn request_history(&mut self, request: HistoryRequest) -> (u64, Message) {
        let id = self.next_id;
        self.next_id += 1;

        let message = self.build_request_message(&request);

        self.pending.push_back(PendingRequest { id, request });

        (id, message)
    }

    /// Build the IRC protocol message for a CHATHISTORY request.
    pub fn build_request_message(&self, request: &HistoryRequest) -> Message {
        match request {
            HistoryRequest::Before {
                target,
                reference,
                limit,
            } => Message::new("CHATHISTORY").with_params(vec![
                "BEFORE".to_string(),
                target.clone(),
                reference.to_param(),
                limit.to_string(),
            ]),
            HistoryRequest::After {
                target,
                reference,
                limit,
            } => Message::new("CHATHISTORY").with_params(vec![
                "AFTER".to_string(),
                target.clone(),
                reference.to_param(),
                limit.to_string(),
            ]),
            HistoryRequest::Between {
                target,
                start_ref,
                end_ref,
                limit,
            } => Message::new("CHATHISTORY").with_params(vec![
                "BETWEEN".to_string(),
                target.clone(),
                start_ref.to_param(),
                end_ref.to_param(),
                limit.to_string(),
            ]),
            HistoryRequest::Around {
                target,
                reference,
                limit,
            } => Message::new("CHATHISTORY").with_params(vec![
                "AROUND".to_string(),
                target.clone(),
                reference.to_param(),
                limit.to_string(),
            ]),
            HistoryRequest::Latest {
                target,
                reference,
                limit,
            } => {
                let ref_param = reference
                    .as_ref()
                    .map(|r| r.to_param())
                    .unwrap_or_else(|| "*".to_string());
                Message::new("CHATHISTORY").with_params(vec![
                    "LATEST".to_string(),
                    target.clone(),
                    ref_param,
                    limit.to_string(),
                ])
            }
        }
    }

    /// Handle a completed history response.
    ///
    /// Removes the oldest pending request from the queue (FIFO correlation)
    /// and returns its ID along with the original request for context.
    /// Returns `None` if there are no pending requests.
    pub fn handle_response(&mut self) -> Option<(u64, HistoryRequest)> {
        self.pending.pop_front().map(|pr| (pr.id, pr.request))
    }

    /// Return the number of pending (unresponded) requests.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Check whether there are any pending requests.
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Clear all pending requests (e.g., on disconnect).
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_before_request() {
        let mgr = ChatHistoryManager::new();
        let request = HistoryRequest::Before {
            target: "#rust".to_string(),
            reference: MessageReference::Timestamp("2025-01-01T00:00:00Z".to_string()),
            limit: 50,
        };

        let msg = mgr.build_request_message(&request);
        assert_eq!(msg.command, "CHATHISTORY");
        assert_eq!(msg.params[0], "BEFORE");
        assert_eq!(msg.params[1], "#rust");
        assert_eq!(msg.params[2], "timestamp=2025-01-01T00:00:00Z");
        assert_eq!(msg.params[3], "50");
    }

    #[test]
    fn test_build_latest_request_with_wildcard() {
        let mgr = ChatHistoryManager::new();
        let request = HistoryRequest::Latest {
            target: "#channel".to_string(),
            reference: None,
            limit: 100,
        };

        let msg = mgr.build_request_message(&request);
        assert_eq!(msg.command, "CHATHISTORY");
        assert_eq!(msg.params[0], "LATEST");
        assert_eq!(msg.params[1], "#channel");
        assert_eq!(msg.params[2], "*");
        assert_eq!(msg.params[3], "100");
    }

    #[test]
    fn test_build_between_request() {
        let mgr = ChatHistoryManager::new();
        let request = HistoryRequest::Between {
            target: "#dev".to_string(),
            start_ref: MessageReference::MsgId("abc123".to_string()),
            end_ref: MessageReference::Timestamp("2025-06-01T12:00:00Z".to_string()),
            limit: 25,
        };

        let msg = mgr.build_request_message(&request);
        assert_eq!(msg.params[0], "BETWEEN");
        assert_eq!(msg.params[1], "#dev");
        assert_eq!(msg.params[2], "msgid=abc123");
        assert_eq!(msg.params[3], "timestamp=2025-06-01T12:00:00Z");
        assert_eq!(msg.params[4], "25");
    }

    #[test]
    fn test_request_queue_lifecycle() {
        let mut mgr = ChatHistoryManager::new();
        assert!(!mgr.has_pending());

        let req1 = HistoryRequest::Latest {
            target: "#a".to_string(),
            reference: None,
            limit: 50,
        };
        let req2 = HistoryRequest::Before {
            target: "#b".to_string(),
            reference: MessageReference::MsgId("xyz".to_string()),
            limit: 20,
        };

        let (id1, _msg1) = mgr.request_history(req1);
        let (id2, _msg2) = mgr.request_history(req2);

        assert_eq!(mgr.pending_count(), 2);
        assert!(mgr.has_pending());
        assert_ne!(id1, id2);

        // FIFO: first response corresponds to first request
        let (resp_id, resp_req) = mgr.handle_response().unwrap();
        assert_eq!(resp_id, id1);
        if let HistoryRequest::Latest { target, .. } = resp_req {
            assert_eq!(target, "#a");
        } else {
            panic!("Expected Latest request");
        }

        assert_eq!(mgr.pending_count(), 1);

        let (resp_id2, _) = mgr.handle_response().unwrap();
        assert_eq!(resp_id2, id2);
        assert_eq!(mgr.pending_count(), 0);
        assert!(mgr.handle_response().is_none());
    }

    #[test]
    fn test_message_reference_parsing() {
        let msgid_ref = MessageReference::parse("msgid=abc123").unwrap();
        assert_eq!(msgid_ref, MessageReference::MsgId("abc123".to_string()));
        assert_eq!(msgid_ref.to_param(), "msgid=abc123");

        let ts_ref = MessageReference::parse("timestamp=2025-01-01T00:00:00Z").unwrap();
        assert_eq!(
            ts_ref,
            MessageReference::Timestamp("2025-01-01T00:00:00Z".to_string())
        );

        assert!(MessageReference::parse("invalid").is_none());
        assert!(MessageReference::parse("").is_none());
    }

    #[test]
    fn test_clear_pending() {
        let mut mgr = ChatHistoryManager::new();

        for i in 0..5 {
            let req = HistoryRequest::Latest {
                target: format!("#{i}"),
                reference: None,
                limit: 10,
            };
            mgr.request_history(req);
        }

        assert_eq!(mgr.pending_count(), 5);
        mgr.clear_pending();
        assert_eq!(mgr.pending_count(), 0);
        assert!(!mgr.has_pending());
    }
}
