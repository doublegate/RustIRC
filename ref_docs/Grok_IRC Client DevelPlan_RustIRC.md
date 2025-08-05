# IRC Client Development Plan: RustIRC

## Introduction

This document outlines a comprehensive plan for developing a modern IRC client named **RustIRC** using the Rust programming language. RustIRC aims to combine the best features from established IRC clients—mIRC (powerful scripting and customization), HexChat (user-friendly GUI and plugin support), and WeeChat (efficiency, scriptability, and buffer management)—while ensuring full compatibility with IRC standards, including IRCv3 extensions. The client will support essential protocols like DCC (Direct Client-to-Client) for file transfers and chats, SASL (Simple Authentication and Security Layer) mechanisms for secure authentication, and cross-platform operation on Linux, macOS, and Windows 10+.

Rust is chosen for its safety, performance, and concurrency features, which are ideal for handling network I/O, multi-threaded operations (e.g., multiple server connections), and memory management without common pitfalls like buffer overflows. The development will prioritize modularity, extensibility, and user privacy, with a focus on open-source principles.

### Project Goals
- **Feature Parity and Innovation**: Blend mIRC's scripting depth, HexChat's intuitive interface, and WeeChat's lightweight efficiency into a single, cohesive client.
- **Standards Compliance**: Fully adhere to RFC 1459 (core IRC), RFC 2812 (IRC client protocol), and IRCv3 specifications (e.g., tags, batch, SASL, cap-negotiate).
- **Cross-Platform Support**: Seamless operation on Linux (via GTK or native), macOS (via Cocoa integration), and Windows 10+ (via WinAPI or cross-platform crates).
- **Security and Performance**: Leverage Rust's ownership model for secure networking; support TLS/SSL for encrypted connections.
- **Extensibility**: Include a plugin system inspired by HexChat and WeeChat, with scripting in Lua or Rust macros.

### Target Audience
- Casual users seeking a simple, modern IRC experience.
- Power users needing advanced scripting and customization.
- Developers interested in contributing to an open-source Rust project.

### Assumptions and Constraints
- Development team: Solo or small team; assume intermediate Rust knowledge.
- Timeline: 6-12 months for MVP to full release, depending on resources.
- Budget: Minimal; rely on free tools and open-source crates.
- No initial mobile support; focus on desktop.

## Requirements Specification

### Functional Requirements
1. **Core IRC Functionality**:
   - Connect to multiple servers simultaneously.
   - Join/part channels, send/receive messages, handle modes (e.g., +o, +b).
   - Nickname management, WHOIS, MOTD, and error handling.

2. **IRCv3 Compliance**:
   - Capability negotiation (CAP REQ/LS).
   - Message tags (e.g., server-time, account-tag).
   - Extended features: away-notify, userhost-in-names, multi-prefix.

3. **DCC Support**:
   - DCC CHAT for private sessions.
   - DCC SEND/RECV for file transfers, with resume support.
   - Passive DCC (DCC SEND with reverse connections for NAT traversal).

4. **SASL Mechanisms**:
   - Support PLAIN, EXTERNAL, SCRAM-SHA-256.
   - Integration with capability negotiation.

5. **User Interface**:
   - GUI mode: Tabbed interface with server/channel trees (like HexChat), customizable themes/colors (like mIRC).
   - Optional TUI (Terminal User Interface) mode for lightweight use (inspired by WeeChat), using crates like ratatui.
   - Features: Input history, auto-completion, URL detection/hyperlinks.

6. **Scripting and Plugins**:
   - Scripting engine: Embed Lua (via rlua crate) for mIRC-like scripts; support aliases, timers, events.
   - Plugin system: Dynamic loading of Rust modules or external scripts (Python/Perl via embeddings, like HexChat).
   - WeeChat-inspired buffers: Merge/split views, relay protocol for remote access.

7. **Additional Features**:
   - Logging: Per-channel/server logs with timestamps (mIRC-style).
   - Notifications: Desktop notifications for mentions/highlights.
   - Themes/Skins: CSS-like styling for GUI.
   - Proxy support: SOCKS/HTTP.
   - Internationalization: UTF-8 support, i18n via fluent crate.

### Non-Functional Requirements
- **Performance**: Handle 100+ channels with low CPU/memory usage; asynchronous I/O via Tokio crate.
- **Security**: Validate inputs to prevent injection; use rustls for TLS.
- **Compatibility**: Build and run on:
  - Linux: x86_64, ARM (e.g., Ubuntu, Fedora).
  - macOS: 11+ (Big Sur+).
  - Windows: 10+ (via MSVC toolchain).
- **Accessibility**: Keyboard navigation, screen reader support.
- **Maintainability**: Modular code with unit/integration tests; >80% test coverage.

### Dependencies (Rust Crates)
- Networking: tokio, tokio-rustls.
- IRC Parsing: irc crate (or custom parser for full control).
- GUI: iced (cross-platform, Rust-native) or gtk-rs for broader compatibility.
- TUI: ratatui + crossterm.
- Scripting: rlua for Lua integration.
- Others: serde for config/serialization, notify for file watching, log for logging.

## Architecture Overview

RustIRC will follow a modular, event-driven architecture:

- **Layers**:
  - **Network Layer**: Handles TCP/TLS connections, IRC message parsing/serialization.
  - **Core Logic Layer**: Manages state (servers, channels, users), event dispatching.
  - **UI Layer**: Abstracts GUI/TUI; uses observer pattern for updates.
  - **Plugin Layer**: Hooks into events (e.g., on_message, on_connect).

- **Data Flow**:
  - Incoming messages → Parser → Event Dispatcher → Plugins/UI.
  - Outgoing: UI/Plugins → Serializer → Network.

- **Concurrency**: Use Tokio for async tasks (e.g., one runtime per connection).
- **Configuration**: TOML files for settings, scripts in a user directory.

Text-based Diagram:
```
[User Input] --> [UI Layer (GUI/TUI)] <--> [Event Dispatcher]
                                      |
                                      v
[Plugins/Scripts] <--> [Core Logic (State Management)]
                                      |
                                      v
[Network Layer (Tokio Async)] <--> [IRC Server]
```

## Phased Implementation Plan

The development is divided into phases with milestones, deliverables, and estimated timelines (assuming full-time development). Each phase includes tasks, risks, and success criteria.

### Phase 1: Research and Planning (2-4 Weeks)
- **Tasks**:
  - Analyze mIRC, HexChat, WeeChat: Document features (e.g., scripting syntax, UI elements).
  - Review IRC specs: RFCs, IRCv3 docs.
  - Select crates: Prototype small network tests.
  - Define MVP: Core connection, basic chat.
  - Set up repo: GitHub with Cargo workspace.
- **Deliverables**: Requirements doc (this), wireframes for UI, initial Cargo.toml.
- **Risks**: Over-scoping features; mitigate by prioritizing core IRC.
- **Milestone**: Approved design; basic project skeleton.

### Phase 2: Core IRC Protocol Implementation (4-6 Weeks)
- **Tasks**:
  - Implement IRC parser/serializer (extend irc crate if needed).
  - Handle connections: Async connect/disconnect, ping/pong.
  - Basic commands: JOIN, PART, PRIVMSG, NICK.
  - Integrate SASL: PLAIN mechanism first.
  - State management: Track channels, users with HashMaps.
- **Deliverables**: CLI prototype for connecting and chatting (no UI yet).
- **Risks**: Parsing edge cases; address with fuzz testing.
- **Milestone**: Successful connection to Freenode/Libera.Chat; unit tests passing.

Example Code Snippet (Rust):
```rust
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn connect(server: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect((server, port)).await?;
    stream.write_all(b"NICK rustirc\r\nUSER rustirc 0 * :Rust IRC\r\n").await?;
    // Read loop for responses...
    Ok(())
}
```

### Phase 3: UI Development (6-8 Weeks)
- **Tasks**:
  - Build GUI: Use iced for cross-platform windows, tabs, input bar.
  - Add TUI fallback: ratatui for terminal mode.
  - Integrate with core: Real-time message display, input handling.
  - Basic customization: Themes via config.
- **Deliverables**: Runnable app with basic chat interface.
- **Risks**: Cross-platform UI bugs; test on all OS early.
- **Milestone**: GUI/TUI rendering messages; cross-OS builds.

### Phase 4: Scripting and Plugins (4-6 Weeks)
- **Tasks**:
  - Embed Lua: Handle events like on_join, on_message.
  - Plugin API: Define traits for extensions.
  - Aliases/Popups: mIRC-inspired syntax in Lua.
  - Buffers: WeeChat-like merging/splitting.
- **Deliverables**: Sample scripts (e.g., auto-greet); plugin loader.
- **Risks**: Security in scripting; sandbox Lua execution.
- **Milestone**: Scripts modifying behavior (e.g., highlight filter).

Example Lua Script Integration:
```rust
use rlua::Lua;

let lua = Lua::new();
lua.context(|ctx| {
    ctx.load("function on_message(msg) print('Received: ' .. msg) end").exec()?;
});
```

### Phase 5: Advanced Features and Protocols (6-8 Weeks)
- **Tasks**:
  - DCC: Implement SEND/RECV, CHAT with file progress.
  - Full IRCv3: Tags, multi-prefix.
  - SASL Extensions: EXTERNAL, SCRAM.
  - Notifications, logging, proxies.
- **Deliverables**: DCC file transfer demo; full standards compliance.
- **Risks**: NAT issues in DCC; use UPnP crate if needed.
- **Milestone**: End-to-end tests with real servers.

### Phase 6: Testing and Optimization (4-6 Weeks)
- **Tasks**:
  - Unit/Integration Tests: cargo test; simulate servers.
  - Cross-Platform Builds: Use GitHub Actions for CI/CD.
  - Performance Tuning: Profile with flamegraph.
  - Security Audit: Check for vulnerabilities.
  - Beta Testing: Release to community for feedback.
- **Deliverables**: Test reports; optimized binaries.
- **Risks**: OS-specific bugs; iterative fixes.
- **Milestone**: 90% test coverage; beta feedback incorporated.

### Phase 7: Release and Maintenance (Ongoing)
- **Tasks**:
  - Packaging: Binaries for Linux (deb/rpm), macOS (dmg), Windows (msi).
  - Documentation: User guide, API docs.
  - Release v1.0: GitHub releases.
  - Post-Release: Bug fixes, feature requests via issues.
- **Deliverables**: Published app; changelog.
- **Milestone**: Stable release; community contributions.

## Tools and Development Environment
- **IDE**: VS Code with rust-analyzer.
- **Build Tools**: Cargo, cross for cross-compilation.
- **Testing**: cargo-test, mockito for network mocks.
- **Version Control**: Git; semantic versioning.
- **CI/CD**: GitHub Actions for builds/tests on Linux/macOS/Windows.

## Risks and Mitigation
- **Technical Debt**: Modular design to refactor easily.
- **Dependency Issues**: Pin crate versions; fallback to custom impl.
- **Community Adoption**: Engage on IRC channels/Reddit for early feedback.

## Conclusion
This phased plan ensures a structured path to a robust, feature-rich IRC client. By leveraging Rust's strengths and drawing from proven clients, RustIRC will offer a superior experience. Next steps: Initiate Phase 1 and prototype the core network layer. For contributions or questions, refer to the project repo.
