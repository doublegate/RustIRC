//! Integration tests for DCC protocol

use rustirc_core::config::DccConfig;
use rustirc_core::dcc::{DccManager, DccRequest};
use std::net::{IpAddr, Ipv4Addr};

#[test]
fn test_dcc_manager_creation() {
    let config = DccConfig::default();
    let mgr = DccManager::new(config);

    let rt = tokio::runtime::Runtime::new().unwrap();
    assert!(rt.block_on(mgr.list_sessions()).is_empty());
}

#[test]
fn test_dcc_request_parsing_send() {
    let request = DccManager::parse_dcc_request("alice", "SEND file.txt 2130706433 1234 5678");
    assert!(request.is_ok());

    if let Ok(DccRequest::Send {
        peer_nick,
        filename,
        address,
        port,
        file_size,
    }) = request
    {
        assert_eq!(peer_nick, "alice");
        assert_eq!(filename, "file.txt");
        assert_eq!(address, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(port, 1234);
        assert_eq!(file_size, 5678);
    } else {
        panic!("Expected DCC SEND request");
    }
}

#[test]
fn test_dcc_request_parsing_chat() {
    let request = DccManager::parse_dcc_request("bob", "CHAT chat 2130706433 1234");
    assert!(request.is_ok());

    if let Ok(DccRequest::Chat {
        peer_nick,
        address,
        port,
    }) = request
    {
        assert_eq!(peer_nick, "bob");
        assert_eq!(address, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(port, 1234);
    } else {
        panic!("Expected DCC CHAT request");
    }
}

#[test]
fn test_dcc_request_parsing_resume() {
    let request = DccManager::parse_dcc_request("carol", "RESUME file.txt 1234 5678");
    assert!(request.is_ok());

    if let Ok(DccRequest::Resume {
        peer_nick,
        filename,
        port,
        position,
    }) = request
    {
        assert_eq!(peer_nick, "carol");
        assert_eq!(filename, "file.txt");
        assert_eq!(port, 1234);
        assert_eq!(position, 5678);
    } else {
        panic!("Expected DCC RESUME request");
    }
}

#[test]
fn test_dcc_ip_conversion() {
    // 127.0.0.1 = 2130706433
    let addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let ip_long = DccManager::ip_to_long(&addr);
    assert_eq!(ip_long, 2130706433);
}

#[test]
fn test_dcc_disabled_config() {
    let config = DccConfig {
        enabled: false,
        ..Default::default()
    };
    let mgr = DccManager::new(config);

    let rt = tokio::runtime::Runtime::new().unwrap();
    assert!(rt.block_on(mgr.list_sessions()).is_empty());
}

#[test]
fn test_dcc_request_parsing_invalid() {
    assert!(DccManager::parse_dcc_request("nick", "not a DCC request").is_err());
    assert!(DccManager::parse_dcc_request("nick", "UNKNOWN foo bar baz").is_err());
    assert!(DccManager::parse_dcc_request("nick", "SEND").is_err());
}
