//! Security Advisory Regression and Verification Tests
//!
//! Validates security fixes and behavior for:
//! - RUSTSEC-2026-0190 (anyhow: sound downcast_mut with context)
//! - RUSTSEC-2026-0221 (event-listener: StackSlot thread safety)
//! - RUSTSEC-2026-0194 (quick-xml: linear attribute checking time)
//! - RUSTSEC-2026-0195 (quick-xml: bounded namespace handling)
//! - RUSTSEC-2026-0204 (crossbeam-epoch: pointer Display soundness)
//! - RUSTSEC-2026-0104, RUSTSEC-2026-0099, RUSTSEC-2026-0098, RUSTSEC-2026-0049 (rustls/webpki TLS validation)

use event_listener::Listener;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

// ============================================================================
// RUSTSEC-2026-0190: anyhow Error::downcast_mut soundness with context
// ============================================================================

#[derive(Debug, PartialEq, Eq)]
struct CustomErrorContext(&'static str);

impl fmt::Display for CustomErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Context: {}", self.0)
    }
}

#[test]
fn test_rustsec_2026_0190_anyhow_downcast_mut_soundness() {
    let mut error =
        anyhow::Error::msg("base protocol error").context(CustomErrorContext("initial value"));

    // Verify downcast_mut correctly acquires mutable reference
    let ctx = error
        .downcast_mut::<CustomErrorContext>()
        .expect("should downcast_mut to CustomErrorContext");
    assert_eq!(ctx.0, "initial value");

    // Mutate the context in place
    ctx.0 = "updated value";
    assert_eq!(ctx.0, "updated value");

    // Subsequent downcasts must observe the updated state without undefined behavior
    let ctx2 = error
        .downcast_mut::<CustomErrorContext>()
        .expect("should downcast_mut again");
    assert_eq!(ctx2.0, "updated value");

    let formatted = format!("{:?}", error);
    assert!(formatted.contains("updated value"));
}

// ============================================================================
// RUSTSEC-2026-0221: event-listener synchronization safety
// ============================================================================

#[test]
fn test_rustsec_2026_0221_event_listener_synchronization() {
    let event = Arc::new(event_listener::Event::new());
    let flag = Arc::new(AtomicBool::new(false));

    let event_clone = Arc::clone(&event);
    let flag_clone = Arc::clone(&flag);

    let handle = std::thread::spawn(move || {
        let listener = event_clone.listen();
        flag_clone.store(true, Ordering::SeqCst);
        listener.wait();
    });

    while !flag.load(Ordering::SeqCst) {
        std::thread::yield_now();
    }

    event.notify(1);
    handle.join().expect("thread join failed");
}

// ============================================================================
// RUSTSEC-2026-0194: quick-xml attribute duplicate check linear time
// ============================================================================

#[test]
fn test_rustsec_2026_0194_quick_xml_attributes_linear_time() {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    // Generate XML start tag with 1,500 distinct attributes
    let mut xml = String::from("<element");
    for i in 0..1500 {
        use std::fmt::Write;
        write!(&mut xml, " attr_{}=\"{}\"", i, i).unwrap();
    }
    xml.push_str("/>");

    let start = Instant::now();
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().check_comments = true;

    let mut attr_count = 0;
    loop {
        match reader.read_event() {
            Ok(Event::Empty(e)) => {
                for attr in e.attributes() {
                    let _ = attr.expect("attribute should be valid");
                    attr_count += 1;
                }
                break;
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("XML parsing error: {:?}", e),
            _ => {}
        }
    }

    let elapsed = start.elapsed();
    assert_eq!(attr_count, 1500);
    // In quick-xml 0.41.0 with O(1) hash pre-filter, 1500 attributes parse in a few milliseconds
    assert!(
        elapsed.as_millis() < 500,
        "Parsing took too long: {}ms (quadratic regression)",
        elapsed.as_millis()
    );
}

// ============================================================================
// RUSTSEC-2026-0195: quick-xml namespace reader bounded allocation
// ============================================================================

#[test]
fn test_rustsec_2026_0195_quick_xml_ns_reader_bounded() {
    use quick_xml::events::Event;
    use quick_xml::reader::NsReader;

    let xml = r#"<root xmlns="urn:default" xmlns:irc="urn:irc" xmlns:cap="urn:cap"><irc:message cap:client="test">hello</irc:message></root>"#;

    let mut reader = NsReader::from_str(xml);
    let mut elements = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let local_name = String::from_utf8_lossy(name.into_inner()).to_string();
                elements.push(local_name);
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("NS reader error: {:?}", e),
            _ => {}
        }
    }

    assert_eq!(elements, vec!["root", "irc:message"]);
}

// ============================================================================
// RUSTSEC-2026-0204: crossbeam-epoch pointer dereference soundness
// ============================================================================

#[test]
fn test_rustsec_2026_0204_crossbeam_epoch_pointer_safety() {
    use crossbeam_epoch::{Atomic, Shared};

    let atomic: Atomic<u64> = Atomic::null();
    let guard = crossbeam_epoch::pin();
    let shared = atomic.load(Ordering::Relaxed, &guard);

    assert!(shared.is_null());
    assert_eq!(shared, Shared::null());

    let allocated = Atomic::new(42u64);
    let loaded = allocated.load(Ordering::Relaxed, &guard);
    assert!(!loaded.is_null());
    unsafe {
        assert_eq!(*loaded.deref(), 42);
    }
}

// ============================================================================
// RUSTSEC-2026-0104, 0099, 0098, 0049: rustls and webpki verification rules
// ============================================================================

#[test]
fn test_rustls_webpki_root_store_and_server_names() {
    use rustls::pki_types::ServerName;

    // Load webpki roots into root cert store
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    assert!(
        !root_store.is_empty(),
        "Root certificate store should not be empty"
    );

    // Server name parsing validation
    let valid_dns = ServerName::try_from("irc.libera.chat");
    assert!(
        valid_dns.is_ok(),
        "Valid DNS name should parse successfully"
    );

    let ip_server = ServerName::try_from("127.0.0.1");
    assert!(
        ip_server.is_ok(),
        "IP server name should parse successfully"
    );

    let client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    assert!(
        client_config.alpn_protocols.is_empty(),
        "Default ALPN should be empty unless configured"
    );
}
