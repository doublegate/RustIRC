//! Security advisory checks and configuration smoke tests.
//!
//! Lockfile minimum-version assertions enforce patched dependencies. Ordinary
//! value assertions do not establish absence of undefined behavior; use Miri for
//! additional checking. The TLS tests below do not validate certificate chains,
//! name constraints, or advisory-specific CRL extensions.
//!
//! Related advisories:
//! - RUSTSEC-2026-0190 (anyhow: sound downcast_mut with context)
//! - RUSTSEC-2026-0221 (event-listener: StackSlot thread safety)
//! - RUSTSEC-2026-0194 (quick-xml: linear attribute checking time)
//! - RUSTSEC-2026-0195 (quick-xml: bounded namespace handling)
//! - RUSTSEC-2026-0204 (crossbeam-epoch: pointer Display/Pointer soundness)
//! - RUSTSEC-2026-0104, RUSTSEC-2026-0099, RUSTSEC-2026-0098, RUSTSEC-2026-0049 (rustls/webpki TLS validation)

use event_listener::Listener;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

/// Checks context mutations and formatting; soundness relies on the patched dependency.
#[test]
fn test_rustsec_2026_0190_anyhow_downcast_mut_soundness() {
    // Note: The undefined behavior under Stacked Borrows was fixed in anyhow commit 6e8c000 (v1.0.103+).
    // This test exercises the exact API sequence (context followed by downcast_mut mutation)
    // to confirm state preservation. Value assertions alone cannot detect undefined behavior.
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

    // Subsequent downcasts must observe the updated state
    let ctx2 = error
        .downcast_mut::<CustomErrorContext>()
        .expect("should downcast_mut again");
    assert_eq!(ctx2.0, "updated value");

    let formatted = format!("{:?}", error);
    assert!(formatted.contains("updated value"));
}

// ============================================================================
// RUSTSEC-2026-0221: event-listener StackSlot thread safety
// ============================================================================

// Negative assertions target StackSlot directly, avoiding a compile-fail test
// that could fail solely because a borrowed listener cannot escape its lifetime.
static_assertions::assert_not_impl_any!(
    event_listener::__private::StackSlot<'static, std::rc::Rc<()>>: Send, Sync
);
static_assertions::assert_impl_all!(
    event_listener::__private::StackSlot<'static, u64>: Send, Sync
);

/// Checks listener notification; the module-level trait assertions cover the advisory.
#[test]
fn test_rustsec_2026_0221_event_listener_stack_slot_safety() {
    // RUSTSEC-2026-0221: StackSlot unconditionally implemented Send/Sync in 5.1.0-5.4.1.
    // In 5.4.2, StackSlot<'_, T> is Send/Sync only when T: Send/Sync.
    // Verify that StackSlot with Send tag satisfies Send + Sync bounds:

    // Exercise the listener! macro (which creates StackSlot) across thread boundaries:
    let event = Arc::new(event_listener::Event::new());
    let ready = Arc::new(AtomicBool::new(false));

    let event_clone = Arc::clone(&event);
    let ready_clone = Arc::clone(&ready);

    let handle = std::thread::spawn(move || {
        event_listener::listener!(event_clone => listener);
        ready_clone.store(true, Ordering::SeqCst);
        listener.wait();
    });

    while !ready.load(Ordering::SeqCst) {
        std::thread::yield_now();
    }

    event.notify(1);
    handle.join().expect("thread join failed");
}

// ============================================================================
// RUSTSEC-2026-0194: quick-xml attribute duplicate check linear scaling
// ============================================================================

/// Checks duplicate detection without a noisy CI wall-clock complexity assertion.
#[test]
fn test_quick_xml_duplicate_attribute_detection() {
    use quick_xml::events::attributes::AttrError;
    use quick_xml::events::BytesStart;

    let mut tag = String::from("element");
    for i in 0..2500 {
        use std::fmt::Write;
        write!(&mut tag, " attr_{i}=\"{i}\"").unwrap();
    }
    tag.push_str(" attr_2499=\"duplicate\"");
    let element = BytesStart::from_content(tag, "element".len());
    let mut attributes = element.attributes();
    for _ in 0..2500 {
        assert!(attributes.next().unwrap().is_ok());
    }
    assert!(matches!(
        attributes.next(),
        Some(Err(AttrError::Duplicated(..)))
    ));
}

/// Fails if any resolved copy of an affected crate falls below its patched floor.
#[test]
fn test_security_advisory_dependency_floors() {
    let lock: toml::Value = toml::from_str(include_str!("../Cargo.lock")).unwrap();
    let packages = lock["package"].as_array().unwrap();
    for (name, minimum) in [
        ("anyhow", (1, 0, 103)),
        ("event-listener", (5, 4, 2)),
        ("quick-xml", (0, 41, 0)),
        ("crossbeam-epoch", (0, 9, 20)),
        ("rustls-webpki", (0, 103, 13)),
    ] {
        let mut found = false;
        for package in packages.iter().filter(|p| p["name"].as_str() == Some(name)) {
            found = true;
            let version = package["version"].as_str().unwrap();
            // Prereleases are deliberately rejected rather than assumed patched.
            let components: Vec<u64> = version.split('.').map(|s| s.parse().unwrap()).collect();
            assert_eq!(components.len(), 3);
            assert!(
                (components[0], components[1], components[2]) >= minimum,
                "{name} {version} is below the patched minimum {minimum:?}"
            );
        }
        assert!(found, "{name} missing from lockfile");
    }
}

// ============================================================================
// RUSTSEC-2026-0195: quick-xml namespace reader bounded allocation
// ============================================================================

/// Rejects a start tag exceeding the default namespace-declaration limit.
#[test]
fn test_rustsec_2026_0195_quick_xml_ns_reader_bounded() {
    use quick_xml::events::Event;
    use quick_xml::reader::NsReader;

    // 1. Valid namespaces within the 256 default limit parse cleanly
    let valid_xml = r#"<root xmlns="urn:default" xmlns:irc="urn:irc" xmlns:cap="urn:cap"><irc:message cap:client="test">hello</irc:message></root>"#;
    let mut reader = NsReader::from_str(valid_xml);
    let mut elements = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let local_name = String::from_utf8_lossy(name.into_inner()).to_string();
                elements.push(local_name);
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Valid NS reader error: {:?}", e),
            _ => {}
        }
    }
    assert_eq!(elements, vec!["root", "irc:message"]);

    // 2. Exceeding the default 256 boundary (257 declarations) must be rejected
    let mut overflow_xml = String::with_capacity(10000);
    overflow_xml.push_str("<root ");
    for i in 0..257 {
        use std::fmt::Write;
        write!(
            &mut overflow_xml,
            "xmlns:ns{}=\"http://example.com/{}\" ",
            i, i
        )
        .unwrap();
    }
    overflow_xml.push_str("><child/></root>");

    let mut overflow_reader = NsReader::from_str(&overflow_xml);
    assert!(matches!(
        overflow_reader.read_event(),
        Err(quick_xml::Error::Namespace(
            quick_xml::name::NamespaceError::TooManyDeclarations(256)
        ))
    ));
}

// ============================================================================
// RUSTSEC-2026-0204: crossbeam-epoch pointer formatting soundness
// ============================================================================

/// Checks null-pointer formatting; run under Miri for additional UB detection.
#[test]
fn test_rustsec_2026_0204_crossbeam_epoch_pointer_safety() {
    use crossbeam_epoch::{Atomic, Shared};

    // RUSTSEC-2026-0204: fmt::Pointer implementation on null Atomic or Shared
    // dereferenced the underlying pointer, causing invalid pointer dereference.
    // Verify that formatting null Atomic and null Shared with {:p} does not crash:
    let atomic: Atomic<u64> = Atomic::null();
    let fmt_atomic = format!("{:p}", atomic);
    assert_eq!(fmt_atomic, format!("{:p}", std::ptr::null::<u64>()));

    let shared: Shared<'_, u64> = Shared::null();
    let fmt_shared = format!("{:p}", shared);
    assert_eq!(
        fmt_shared,
        format!("{:p}", std::ptr::null::<u64>()),
        "Null Shared pointer must format as null pointer address without dereferencing"
    );
}

// ============================================================================
// RUSTSEC-2026-0104, 0099, 0098, 0049: rustls and webpki verification rules
// ============================================================================

/// Smoke-tests TLS configuration, not certificate validation or revocation.
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

/// Rejects invalid DER; this is not the onlySomeReasons advisory reproducer.
#[test]
fn test_webpki_invalid_crl_der_rejected() {
    // These buffers fail before parsing any IssuingDistributionPoint extension.
    let empty_der = b"";
    let crl_result = webpki::BorrowedCertRevocationList::from_der(empty_der);
    assert!(crl_result.is_err(), "Empty DER must return Err, not panic");

    let malformed_der = &[0x30, 0x05, 0x03, 0x00, 0x00];
    let crl_malformed = webpki::BorrowedCertRevocationList::from_der(malformed_der);
    assert!(
        crl_malformed.is_err(),
        "Malformed CRL DER must return Err without panic"
    );
}
