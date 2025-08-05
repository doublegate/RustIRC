# RustIRC Testing Strategy

## Overview

This document outlines the comprehensive testing strategy for RustIRC, covering unit tests, integration tests, end-to-end tests, performance testing, security testing, and cross-platform validation. Our goal is to achieve 90%+ code coverage while ensuring reliability, performance, and security across all supported platforms.

## Testing Philosophy

### Core Principles

1. **Test-Driven Development (TDD)**: Write tests before implementation
2. **Comprehensive Coverage**: Unit, integration, and end-to-end tests
3. **Continuous Testing**: Tests run on every commit
4. **Performance Regression Prevention**: Benchmark critical paths
5. **Security-First**: Include security tests for all input handling
6. **Cross-Platform Validation**: Test on Windows, macOS, and Linux

### Testing Pyramid

```
        /\
       /E2E\      <- End-to-end tests (10%)
      /------\
     /  Integ  \   <- Integration tests (30%)
    /----------\
   /    Unit     \  <- Unit tests (60%)
  /--------------\
```

## Test Infrastructure

### Testing Frameworks

```toml
# Cargo.toml
[dev-dependencies]
# Core testing
tokio-test = "0.4"
mockall = "0.12"
proptest = "1.4"
criterion = "0.5"

# Test utilities
pretty_assertions = "1.4"
test-case = "3.3"
rstest = "0.18"
wiremock = "0.6"

# Security testing
cargo-fuzz = "0.11"
```

### Mock IRC Server

```rust
// tests/common/mock_irc_server.rs
pub struct MockIrcServer {
    listener: TcpListener,
    expectations: Vec<Expectation>,
    responses: HashMap<String, String>,
}

impl MockIrcServer {
    pub async fn new() -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        Ok(Self {
            listener,
            expectations: Vec::new(),
            responses: HashMap::new(),
        })
    }
    
    pub fn expect(&mut self, pattern: &str) -> &mut Self {
        self.expectations.push(Expectation::new(pattern));
        self
    }
    
    pub fn respond(&mut self, pattern: &str, response: &str) -> &mut Self {
        self.responses.insert(pattern.to_string(), response.to_string());
        self
    }
    
    pub async fn run(self) {
        tokio::spawn(async move {
            while let Ok((stream, _)) = self.listener.accept().await {
                // Handle mock connection
            }
        });
    }
}
```

### Test Environment

```rust
// tests/common/test_env.rs
pub struct TestEnv {
    pub temp_dir: TempDir,
    pub config: ClientConfig,
    pub mock_server: MockIrcServer,
}

impl TestEnv {
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let mock_server = MockIrcServer::new().await?;
        
        let config = ClientConfig {
            data_dir: temp_dir.path().to_path_buf(),
            server: ServerConfig {
                address: mock_server.address(),
                nick: "testbot".to_string(),
                // ...
            },
            // ...
        };
        
        Ok(Self { temp_dir, config, mock_server })
    }
}
```

## Unit Testing

### Protocol Parser Tests

```rust
#[cfg(test)]
mod parser_tests {
    use super::*;
    use proptest::prelude::*;
    use test_case::test_case;
    
    #[test_case("PRIVMSG #channel :Hello" => Ok(IrcMessage {
        command: "PRIVMSG".to_string(),
        params: vec!["#channel".to_string(), "Hello".to_string()],
        ..Default::default()
    }); "basic privmsg")]
    #[test_case("@time=2021-01-01T00:00:00.000Z :nick!user@host PRIVMSG #channel :Hi" => Ok(_); "with tags")]
    fn test_parse_message(input: &str) -> Result<IrcMessage> {
        parse_irc_message(input)
    }
    
    proptest! {
        #[test]
        fn test_parse_doesnt_crash(s in "\\PC*") {
            let _ = parse_irc_message(&s);
        }
        
        #[test]
        fn test_parse_roundtrip(msg in arb_irc_message()) {
            let serialized = msg.to_string();
            let parsed = parse_irc_message(&serialized)?;
            prop_assert_eq!(parsed, msg);
        }
    }
    
    #[test]
    fn test_utf8_handling() {
        let messages = vec![
            "PRIVMSG #channel :Hello 👋",
            "PRIVMSG #channel :Здравствуйте",
            "PRIVMSG #channel :こんにちは",
            "PRIVMSG #channel :🦀 Rust",
        ];
        
        for msg in messages {
            let parsed = parse_irc_message(msg).unwrap();
            assert!(parsed.params[1].chars().all(|c| c.is_char()));
        }
    }
}
```

### State Management Tests

```rust
#[cfg(test)]
mod state_tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    
    #[tokio::test]
    async fn test_concurrent_state_updates() {
        let state = Arc::new(RwLock::new(IrcState::new()));
        let mut handles = vec![];
        
        // Spawn 100 concurrent tasks
        for i in 0..100 {
            let state_clone = Arc::clone(&state);
            let handle = tokio::spawn(async move {
                let mut state = state_clone.write().await;
                state.add_channel(ServerId(0), &format!("#channel{}", i));
            });
            handles.push(handle);
        }
        
        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify state
        let state = state.read().await;
        assert_eq!(state.channel_count(ServerId(0)), 100);
    }
    
    #[test]
    fn test_user_tracking() {
        let mut state = ChannelState::new("#test");
        
        // Add users
        state.add_user("alice", UserModes::default());
        state.add_user("bob", UserModes::from_str("+o"));
        state.add_user("charlie", UserModes::from_str("+v"));
        
        // Verify
        assert_eq!(state.user_count(), 3);
        assert!(state.is_op("bob"));
        assert!(state.has_voice("charlie"));
        assert!(!state.is_op("alice"));
    }
}
```

### Script Engine Tests

```rust
#[cfg(test)]
mod script_tests {
    use super::*;
    
    #[test]
    fn test_lua_sandbox() {
        let engine = LuaEngine::new(true).unwrap();
        
        // These should fail
        let dangerous = vec![
            "os.execute('rm -rf /')",
            "io.open('/etc/passwd', 'r')",
            "require('os').exit()",
            "loadfile('/etc/passwd')",
        ];
        
        for code in dangerous {
            assert!(engine.execute(code).is_err(), 
                "Dangerous code should be blocked: {}", code);
        }
        
        // These should work
        let safe = vec![
            "return 2 + 2",
            "local t = {} for i=1,10 do t[i] = i end return #t",
            "return string.upper('hello')",
        ];
        
        for code in safe {
            assert!(engine.execute(code).is_ok(),
                "Safe code should work: {}", code);
        }
    }
    
    #[test]
    fn test_python_sandbox() {
        let engine = PythonEngine::new(true).unwrap();
        
        Python::with_gil(|py| {
            // Test restricted imports
            let result = py.eval("__import__('os')", None, None);
            assert!(result.is_err());
            
            // Test safe operations
            let result = py.eval("2 + 2", None, None).unwrap();
            assert_eq!(result.extract::<i32>().unwrap(), 4);
        });
    }
}
```

## Integration Testing

### Connection Flow Tests

```rust
// tests/integration/connection_test.rs
#[tokio::test]
async fn test_full_connection_flow() {
    let env = TestEnv::new().await.unwrap();
    
    // Setup mock server expectations
    env.mock_server
        .expect("CAP LS 302")
        .respond("CAP * LS :sasl message-tags server-time")
        .expect("CAP REQ :sasl message-tags server-time")
        .respond("CAP * ACK :sasl message-tags server-time")
        .expect("NICK testbot")
        .expect("USER testbot 0 * :Test Bot")
        .respond(":server 001 testbot :Welcome")
        .respond(":server 376 testbot :End of MOTD")
        .expect("CAP END");
    
    env.mock_server.run().await;
    
    // Connect client
    let mut client = Client::new(env.config);
    let server_id = client.connect().await.unwrap();
    
    // Wait for registration
    tokio::time::timeout(Duration::from_secs(5), async {
        while !client.is_registered(server_id) {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }).await.unwrap();
    
    assert!(client.is_connected(server_id));
    assert_eq!(client.current_nick(server_id), Some("testbot"));
}

#[tokio::test]
async fn test_sasl_authentication() {
    let env = TestEnv::new().await.unwrap();
    
    env.mock_server
        .expect("CAP REQ :sasl")
        .respond("CAP * ACK :sasl")
        .expect("AUTHENTICATE PLAIN")
        .respond("AUTHENTICATE +")
        .expect("AUTHENTICATE dGVzdGJvdAB0ZXN0Ym90AHBhc3N3b3Jk") // base64: testbot\0testbot\0password
        .respond(":server 900 testbot testbot!testbot@host testbot :You are now logged in")
        .respond(":server 903 testbot :SASL authentication successful");
    
    // Test SASL flow
    let result = perform_sasl_auth(&mut client, "PLAIN", "testbot", "password").await;
    assert!(result.is_ok());
}
```

### Multi-Server Tests

```rust
#[tokio::test]
async fn test_multiple_server_connections() {
    let mut client = Client::new(ClientConfig::default());
    
    // Connect to multiple servers
    let server1 = client.connect(ServerConfig {
        address: "127.0.0.1:6667".parse().unwrap(),
        nick: "bot1".to_string(),
        ..Default::default()
    }).await.unwrap();
    
    let server2 = client.connect(ServerConfig {
        address: "127.0.0.1:6668".parse().unwrap(),
        nick: "bot2".to_string(),
        ..Default::default()
    }).await.unwrap();
    
    // Verify separate states
    assert_ne!(server1, server2);
    assert_eq!(client.current_nick(server1), Some("bot1"));
    assert_eq!(client.current_nick(server2), Some("bot2"));
    
    // Test message routing
    client.send_message(server1, "#chan1", "Hello from server1").await.unwrap();
    client.send_message(server2, "#chan2", "Hello from server2").await.unwrap();
}
```

### DCC Transfer Tests

```rust
#[tokio::test]
async fn test_dcc_file_transfer() {
    let env = TestEnv::new().await.unwrap();
    
    // Create test file
    let test_file = env.temp_dir.path().join("test.txt");
    std::fs::write(&test_file, b"Hello, DCC!").unwrap();
    
    // Setup DCC listener
    let dcc_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dcc_addr = dcc_listener.local_addr().unwrap();
    
    // Initiate transfer
    let transfer_id = client.dcc_send(server_id, "peer", &test_file).await.unwrap();
    
    // Simulate peer accepting
    let accept_task = tokio::spawn(async move {
        let (mut stream, _) = dcc_listener.accept().await.unwrap();
        let mut received = Vec::new();
        stream.read_to_end(&mut received).await.unwrap();
        
        // Send acknowledgment
        let ack = (received.len() as u32).to_be_bytes();
        stream.write_all(&ack).await.unwrap();
        
        received
    });
    
    // Wait for completion
    let received = accept_task.await.unwrap();
    assert_eq!(received, b"Hello, DCC!");
}

#[tokio::test]
async fn test_dcc_resume() {
    // Test partial transfer and resume
    let partial_file = create_partial_file(512);
    let resume_result = client.dcc_resume(transfer_id, 512).await;
    assert!(resume_result.is_ok());
    
    // Verify resumed from correct position
    let final_size = get_file_size(&partial_file).await;
    assert_eq!(final_size, 1024); // Original size
}
```

## End-to-End Testing

### GUI Testing

```rust
// tests/e2e/gui_test.rs
use iced_test::TestApplication;

#[test]
fn test_gui_connection_workflow() {
    let mut app = TestApplication::new(RustIrcGui::new());
    
    // Open connection dialog
    app.click_menu("Server", "Connect...");
    assert!(app.has_dialog("Connect to Server"));
    
    // Fill connection details
    app.fill_input("server_address", "irc.libera.chat");
    app.fill_input("nick", "testuser");
    app.fill_input("channels", "#rust");
    app.check_box("use_tls", true);
    
    // Connect
    app.click_button("Connect");
    
    // Verify connection
    app.wait_for(|state| state.is_connected(), Duration::from_secs(10));
    assert!(app.has_tab("libera.chat"));
    assert!(app.has_channel_tab("#rust"));
}

#[test]
fn test_gui_messaging() {
    let mut app = setup_connected_app();
    
    // Send message
    app.focus_input("message_input");
    app.type_text("Hello, world!");
    app.press_key(Key::Enter);
    
    // Verify message appears
    assert!(app.has_message("testuser", "Hello, world!"));
    
    // Test formatting
    app.type_text("/me waves");
    app.press_key(Key::Enter);
    assert!(app.has_action("testuser", "waves"));
}
```

### TUI Testing

```rust
// tests/e2e/tui_test.rs
use ratatui::test::TestBackend;

#[tokio::test]
async fn test_tui_navigation() {
    let backend = TestBackend::new(80, 24);
    let mut app = TuiApp::new(backend);
    
    // Initial state
    app.render();
    assert!(app.is_showing_server_list());
    
    // Navigate with keyboard
    app.press_key(KeyCode::Tab);
    assert!(app.is_showing_channel_list());
    
    app.press_key(KeyCode::Tab);
    assert!(app.is_showing_message_view());
    
    // Test commands
    app.type_command("/join #test");
    app.press_key(KeyCode::Enter);
    
    app.wait_for_channel("#test");
    assert!(app.current_channel() == Some("#test"));
}

#[test]
fn test_tui_mouse_support() {
    let mut app = setup_tui_app();
    
    // Click on channel
    app.click_at(5, 10); // Channel list area
    assert_eq!(app.selected_channel(), Some("#rust"));
    
    // Scroll messages
    app.scroll_at(40, 12, ScrollDirection::Up);
    assert!(app.message_offset() > 0);
}
```

## Performance Testing

### Benchmarks

```rust
// benches/performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_message_parsing(c: &mut Criterion) {
    let messages = vec![
        "PRIVMSG #channel :Simple message",
        "@time=2021-01-01T00:00:00Z;msgid=123 :nick!user@host PRIVMSG #channel :Tagged message",
        "@badge-info=;badges=moderator/1 :nick!user@host PRIVMSG #channel :Many tags message",
    ];
    
    let mut group = c.benchmark_group("message_parsing");
    
    for (i, msg) in messages.iter().enumerate() {
        group.bench_with_input(BenchmarkId::from_parameter(i), msg, |b, msg| {
            b.iter(|| {
                let parsed = parse_irc_message(black_box(msg));
                black_box(parsed);
            });
        });
    }
    
    group.finish();
}

fn benchmark_state_operations(c: &mut Criterion) {
    c.bench_function("add_100_channels", |b| {
        b.iter_batched(
            || IrcState::new(),
            |mut state| {
                for i in 0..100 {
                    state.add_channel(ServerId(0), &format!("#channel{}", i));
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    c.bench_function("user_list_update", |b| {
        let mut state = create_populated_state();
        b.iter(|| {
            state.update_user_list(
                ServerId(0),
                "#channel",
                (0..100).map(|i| format!("user{}", i)).collect(),
            );
        });
    });
}

criterion_group!(benches, benchmark_message_parsing, benchmark_state_operations);
criterion_main!(benches);
```

### Load Testing

```rust
// tests/load/stress_test.rs
#[tokio::test]
async fn test_high_message_rate() {
    let mut client = Client::new(ClientConfig::default());
    let stats = Arc::new(Mutex::new(MessageStats::default()));
    
    // Setup message counter
    let stats_clone = Arc::clone(&stats);
    client.on_message(move |_, _, _, _| {
        let mut stats = stats_clone.lock().unwrap();
        stats.count += 1;
    });
    
    // Generate high message rate
    let start = Instant::now();
    for i in 0..10000 {
        client.handle_raw_message(&format!(
            ":user{} PRIVMSG #channel :Message {}",
            i % 100, i
        )).await.unwrap();
    }
    let duration = start.elapsed();
    
    // Verify performance
    let stats = stats.lock().unwrap();
    let messages_per_second = stats.count as f64 / duration.as_secs_f64();
    
    assert!(messages_per_second > 1000.0, 
        "Should handle >1000 messages/second, got {}", messages_per_second);
}

#[tokio::test]
async fn test_many_channels() {
    let mut client = Client::new(ClientConfig::default());
    
    // Join many channels
    for i in 0..500 {
        client.join(server_id, &format!("#channel{}", i)).await.unwrap();
    }
    
    // Measure memory usage
    let memory_before = get_process_memory();
    
    // Populate with users
    for i in 0..500 {
        let users: Vec<String> = (0..50).map(|j| format!("user{}", j)).collect();
        client.update_channel_users(server_id, &format!("#channel{}", i), users).await;
    }
    
    let memory_after = get_process_memory();
    let memory_per_channel = (memory_after - memory_before) / 500;
    
    assert!(memory_per_channel < 100 * 1024, // 100KB per channel max
        "Memory usage too high: {} bytes per channel", memory_per_channel);
}
```

## Security Testing

### Input Validation

```rust
// tests/security/input_validation_test.rs
#[test]
fn test_message_length_limits() {
    // IRC message limit is 512 bytes including CRLF
    let long_message = "A".repeat(600);
    let result = validate_irc_message(&long_message);
    assert!(result.is_err());
    
    // Should truncate gracefully
    let truncated = truncate_message(&long_message);
    assert!(truncated.len() <= 510); // Leave room for CRLF
}

#[test]
fn test_null_byte_injection() {
    let messages = vec![
        "PRIVMSG #channel :Hello\0PRIVMSG #admin :Injected",
        "PRIVMSG #channel\0 :Hidden command",
        "PRIVMSG #cha\0nnel :Message",
    ];
    
    for msg in messages {
        let result = validate_irc_message(msg);
        assert!(result.is_err(), "Should reject null bytes: {:?}", msg);
    }
}

#[test]
fn test_unicode_handling() {
    let test_cases = vec![
        ("PRIVMSG #channel :Hello 👋", true),  // Valid emoji
        ("PRIVMSG #channel :Hello \u{202E}world", false), // RTL override
        ("PRIVMSG #channel :Test\u{0000}", false), // Null
        ("PRIVMSG #channel :Line1\nLine2", false), // Newline injection
    ];
    
    for (msg, should_pass) in test_cases {
        let result = validate_irc_message(msg);
        assert_eq!(result.is_ok(), should_pass, "Message: {}", msg);
    }
}
```

### CTCP Flood Protection

```rust
#[tokio::test]
async fn test_ctcp_flood_protection() {
    let mut client = Client::new(ClientConfig::default());
    let attacker = "attacker!user@host";
    
    // Send many CTCP requests rapidly
    for i in 0..100 {
        client.handle_ctcp(attacker, "VERSION", None).await;
    }
    
    // Verify rate limiting kicked in
    let responses = client.get_ctcp_responses();
    assert!(responses.len() < 10, "Should rate limit CTCP responses");
    
    // Verify attacker is temporarily ignored
    tokio::time::sleep(Duration::from_millis(100)).await;
    client.handle_ctcp(attacker, "PING", Some("12345")).await;
    
    let new_responses = client.get_ctcp_responses();
    assert_eq!(responses.len(), new_responses.len(), 
        "Should ignore flood attacker");
}
```

### Script Sandboxing

```rust
#[test]
fn test_lua_sandbox_escapes() {
    let engine = LuaEngine::new(true).unwrap();
    
    // Try various escape attempts
    let escapes = vec![
        // Try to access restricted functions through metatable
        r#"
        local mt = getmetatable("")
        if mt and mt.__index then
            return mt.__index.gsub("test", ".", os.execute)
        end
        "#,
        
        // Try to load bytecode
        r#"
        local bytecode = string.dump(function() os.execute("ls") end)
        return load(bytecode)()
        "#,
        
        // Try to access debug library
        r#"
        local function get_upvalue(func, name)
            local i = 1
            while true do
                local n, v = debug.getupvalue(func, i)
                if not n then break end
                if n == name then return v end
                i = i + 1
            end
        end
        "#,
    ];
    
    for escape in escapes {
        let result = engine.execute(escape);
        assert!(result.is_err() || !result.unwrap().contains("execute"),
            "Escape attempt should fail: {}", escape);
    }
}

#[test]
fn test_python_sandbox_escapes() {
    let engine = PythonEngine::new(true).unwrap();
    
    Python::with_gil(|py| {
        let escapes = vec![
            // Try to import through __builtins__
            "__builtins__.__import__('os')",
            
            // Try to access through subclasses
            "().__class__.__bases__[0].__subclasses__()",
            
            // Try to execute through eval
            "eval('__import__(\"os\").system(\"ls\")')",
            
            // Try to access file objects
            "open('/etc/passwd', 'r')",
        ];
        
        for escape in escapes {
            let result = py.eval(escape, None, None);
            assert!(result.is_err(), 
                "Escape attempt should fail: {}", escape);
        }
    });
}
```

## Fuzzing

### Protocol Fuzzing

```rust
// fuzz/fuzz_targets/protocol_fuzzer.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Fuzz the parser
        let _ = parse_irc_message(s);
        
        // Fuzz message validation
        let _ = validate_irc_message(s);
        
        // Fuzz UTF-8 handling
        let _ = sanitize_message(s);
    }
});
```

### Script Fuzzing

```rust
// fuzz/fuzz_targets/script_fuzzer.rs
fuzz_target!(|data: &[u8]| {
    if let Ok(script) = std::str::from_utf8(data) {
        // Fuzz Lua engine
        let lua_engine = LuaEngine::new(true).unwrap();
        let _ = lua_engine.execute(script);
        
        // Fuzz Python engine
        Python::with_gil(|py| {
            let _ = py.eval(script, None, None);
        });
    }
});
```

## Cross-Platform Testing

### Platform Test Matrix

```yaml
# .github/workflows/cross-platform.yml
name: Cross Platform Tests

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta, nightly]
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: macos-latest
            target: x86_64-apple-darwin
    
    runs-on: ${{ matrix.os }}
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        target: ${{ matrix.target }}
        override: true
    
    - name: Run tests
      run: cargo test --all-features --target ${{ matrix.target }}
    
    - name: Run platform-specific tests
      run: cargo test --test platform_${{ matrix.os }} --target ${{ matrix.target }}
```

### Platform-Specific Tests

```rust
// tests/platform_windows.rs
#![cfg(windows)]

#[test]
fn test_windows_paths() {
    let config_path = get_config_path();
    assert!(config_path.to_str().unwrap().contains("AppData"));
    
    // Test UNC paths
    let unc_path = r"\\server\share\file.txt";
    assert!(is_valid_path(unc_path));
}

#[test]
fn test_windows_line_endings() {
    let message = "Hello\r\nWorld";
    let normalized = normalize_line_endings(message);
    assert_eq!(normalized, "Hello\nWorld");
}

// tests/platform_macos.rs
#![cfg(target_os = "macos")]

#[test]
fn test_macos_keychain() {
    let keychain = Keychain::new("com.rustirc.test");
    
    // Store credential
    keychain.set_password("test_server", "test_user", "test_pass").unwrap();
    
    // Retrieve credential
    let password = keychain.get_password("test_server", "test_user").unwrap();
    assert_eq!(password, "test_pass");
    
    // Clean up
    keychain.delete_password("test_server", "test_user").unwrap();
}

// tests/platform_linux.rs
#![cfg(target_os = "linux")]

#[test]
fn test_xdg_directories() {
    let config_dir = get_config_dir();
    assert!(config_dir.to_str().unwrap().contains(".config"));
    
    let data_dir = get_data_dir();
    assert!(data_dir.to_str().unwrap().contains(".local/share"));
}
```

## Coverage Requirements

### Code Coverage Goals

- Overall: 90%+
- Core modules: 95%+
- Network layer: 85%+
- UI components: 80%+
- Script engines: 90%+
- Platform code: 75%+

### Coverage Reporting

```toml
# .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
runner = "cargo-tarpaulin"

[alias]
coverage = "tarpaulin --out Html --output-dir coverage"
coverage-ci = "tarpaulin --out Xml --output-dir coverage --print-summary"
```

## Test Organization

### Directory Structure

```
tests/
├── common/
│   ├── mod.rs
│   ├── mock_irc_server.rs
│   └── test_env.rs
├── unit/
│   ├── parser_test.rs
│   ├── state_test.rs
│   └── script_test.rs
├── integration/
│   ├── connection_test.rs
│   ├── channel_test.rs
│   └── dcc_test.rs
├── e2e/
│   ├── gui_test.rs
│   └── tui_test.rs
├── security/
│   ├── input_validation_test.rs
│   └── sandbox_test.rs
├── platform/
│   ├── windows_test.rs
│   ├── macos_test.rs
│   └── linux_test.rs
└── load/
    └── stress_test.rs

benches/
├── parser_bench.rs
├── state_bench.rs
└── render_bench.rs

fuzz/
└── fuzz_targets/
    ├── protocol_fuzzer.rs
    └── script_fuzzer.rs
```

## Continuous Integration

### CI Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Check formatting
      run: cargo fmt -- --check
    - name: Clippy
      run: cargo clippy -- -D warnings
  
  test:
    needs: check
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
    - uses: actions/checkout@v3
    - name: Run tests
      run: cargo test --all-features
    - name: Generate coverage
      if: matrix.os == 'ubuntu-latest'
      run: cargo coverage-ci
    - name: Upload coverage
      if: matrix.os == 'ubuntu-latest'
      uses: codecov/codecov-action@v3
  
  security:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Security audit
      run: cargo audit
    - name: Run fuzzer
      run: |
        cargo install cargo-fuzz
        cargo fuzz run protocol_fuzzer -- -max_total_time=60
  
  benchmark:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Run benchmarks
      run: cargo bench
    - name: Upload results
      uses: benchmark-action/github-action-benchmark@v1
```

## Test Guidelines

### Writing Good Tests

1. **Test One Thing**: Each test should verify a single behavior
2. **Descriptive Names**: Use clear, descriptive test names
3. **Arrange-Act-Assert**: Structure tests clearly
4. **Test Edge Cases**: Include boundary conditions
5. **Test Error Cases**: Verify error handling
6. **Use Test Fixtures**: Share setup code appropriately
7. **Avoid Time Dependencies**: Use controlled time in tests
8. **Mock External Dependencies**: Don't rely on network/filesystem

### Test Maintenance

- Run tests before every commit
- Fix broken tests immediately
- Update tests when changing behavior
- Remove obsolete tests
- Keep tests fast and deterministic
- Review test coverage regularly

## Debugging Tests

### Test Utilities

```rust
// Enable detailed logging in tests
#[test]
fn test_with_logging() {
    let _ = env_logger::builder()
        .is_test(true)
        .try_init();
    
    // Your test code
    log::debug!("Debug information");
}

// Capture output in tests
#[test]
fn test_output_capture() {
    let output = capture_output(|| {
        println!("Test output");
    });
    
    assert!(output.contains("Test output"));
}

// Test with timeout
#[tokio::test(flavor = "multi_thread")]
#[timeout(Duration::from_secs(5))]
async fn test_with_timeout() {
    // Test that must complete within 5 seconds
}
```

### Common Issues

1. **Flaky Tests**: Use deterministic time, avoid race conditions
2. **Port Conflicts**: Use port 0 for automatic assignment
3. **File System**: Use temp directories, clean up after tests
4. **Async Issues**: Ensure proper task spawning and joining
5. **Memory Leaks**: Run tests under valgrind/ASAN