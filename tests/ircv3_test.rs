//! Integration tests for IRCv3 extensions

use rustirc_core::batch::{BatchManager, BatchType};
use rustirc_core::chathistory::{ChatHistoryManager, HistoryRequest, MessageReference};
use rustirc_core::flood::FloodProtector;
use rustirc_protocol::message::Tag;
use rustirc_protocol::Message;

#[test]
fn test_batch_lifecycle() {
    let mut mgr = BatchManager::new();

    // Start a batch
    let start_msg = Message::new("BATCH").with_params(vec![
        "+ref1".to_string(),
        "chathistory".to_string(),
        "#channel".to_string(),
    ]);

    mgr.handle_batch_start(&start_msg).unwrap();
    assert!(mgr.is_in_batch("ref1"));

    // Add messages to the batch
    let msg1 = Message::new("PRIVMSG")
        .with_params(vec!["#channel".to_string(), "message 1".to_string()])
        .with_tags(vec![Tag::from_raw("batch", Some("ref1"))]);
    mgr.add_message(&msg1);

    let msg2 = Message::new("PRIVMSG")
        .with_params(vec!["#channel".to_string(), "message 2".to_string()])
        .with_tags(vec![Tag::from_raw("batch", Some("ref1"))]);
    mgr.add_message(&msg2);

    // End the batch
    let end_msg = Message::new("BATCH").with_params(vec!["-ref1".to_string()]);
    let batch = mgr.handle_batch_end(&end_msg).unwrap();

    assert_eq!(batch.batch_type, BatchType::ChatHistory);
    assert_eq!(batch.messages.len(), 2);
}

#[test]
fn test_message_tag_helpers() {
    let msg = Message::new("PRIVMSG")
        .with_params(vec!["#test".to_string(), "hello".to_string()])
        .with_tags(vec![
            Tag::new("time", Some("2025-01-01T00:00:00Z")),
            Tag::new("msgid", Some("abc123")),
            Tag::new("batch", Some("ref42")),
        ]);

    assert!(msg.has_tag("time"));
    assert!(msg.has_tag("msgid"));
    assert!(!msg.has_tag("nonexistent"));

    assert_eq!(msg.get_time(), Some("2025-01-01T00:00:00Z".to_string()));
    assert_eq!(msg.get_msgid(), Some("abc123".to_string()));
    assert_eq!(msg.get_batch(), Some("ref42".to_string()));
}

#[test]
fn test_chathistory_request_building() {
    let mut mgr = ChatHistoryManager::new();

    let request = HistoryRequest::Latest {
        target: "#channel".to_string(),
        reference: None,
        limit: 50,
    };

    let (id, _msg) = mgr.request_history(request);
    assert_eq!(id, 1);

    let before_request = HistoryRequest::Before {
        target: "#channel".to_string(),
        reference: MessageReference::Timestamp("2025-01-01T00:00:00Z".to_string()),
        limit: 100,
    };
    let msg = mgr.build_request_message(&before_request);

    assert_eq!(msg.command, "CHATHISTORY");
    assert_eq!(msg.params[0], "BEFORE");
    assert_eq!(msg.params[1], "#channel");
}

#[test]
fn test_message_reference_parsing() {
    let msgid = MessageReference::parse("msgid=abc123");
    assert_eq!(msgid, Some(MessageReference::MsgId("abc123".to_string())));

    let ts = MessageReference::parse("timestamp=2025-01-01T00:00:00Z");
    assert_eq!(
        ts,
        Some(MessageReference::Timestamp(
            "2025-01-01T00:00:00Z".to_string()
        ))
    );

    assert_eq!(MessageReference::parse("invalid"), None);
}

#[test]
fn test_flood_protection_burst() {
    let mut flood = FloodProtector::new(3, 1.0, 10);

    // Should allow burst of 3
    assert!(flood.try_send());
    assert!(flood.try_send());
    assert!(flood.try_send());
    // 4th should be rejected
    assert!(!flood.try_send());
}

#[test]
fn test_flood_protection_queue() {
    let mut flood = FloodProtector::new(1, 1.0, 5);

    assert!(flood.try_send()); // Use the one token

    // Queue a message
    assert!(flood.enqueue("PRIVMSG #test :hello".to_string()));
    assert!(flood.enqueue("PRIVMSG #test :world".to_string()));
}
