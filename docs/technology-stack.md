# RustIRC Technology Stack

## Core Technologies

### Programming Language
- **Rust** (Edition 2021)
  - Memory safety without garbage collection
  - Excellent concurrency primitives
  - Strong type system and error handling
  - Cross-platform compilation
  - Active ecosystem with cargo

### Async Runtime
- **Tokio** (v1.x)
  - Industry-standard async runtime
  - Multi-threaded work-stealing scheduler
  - Comprehensive async I/O primitives
  - Timer and scheduling utilities
  - Excellent performance characteristics

## Networking

### TLS/SSL
- **rustls** (via tokio-rustls)
  - Pure Rust TLS implementation
  - No OpenSSL dependency
  - Modern cipher suites
  - Certificate validation
  - SNI support

### IRC Protocol
- **Custom Implementation**
  - Full RFC 1459/2812 compliance
  - Complete IRCv3 support
  - Zero-copy parsing where possible
  - Robust error handling

## User Interface

### GUI Framework
- **Primary**: Dioxus (v0.6)
  - React-like component architecture
  - Cross-platform desktop with WebView
  - Virtual DOM with efficient diffing
  - Hot reload development experience
  - RSX macro for JSX-like syntax
  - Modern hooks-based state management
  - CSS-in-Rust styling with runtime themes
  - Native rendering or WebView options

- **Alternative**: Iced (v0.13.1) [Previous Implementation]
  - Pure Rust widget system
  - Elm-style architecture
  - Available on main branch for comparison

### TUI Framework
- **ratatui** (v0.26+)
  - Immediate mode terminal UI
  - Cross-platform terminal support
  - Rich widget library
  - Mouse support
  - Excellent documentation
  - *Note: Unchanged for Dioxus branch*

## Data & Configuration

### Serialization
- **serde** (v1.0)
  - De facto standard for Rust
  - Derive macros for easy use
  - Support for many formats
  - Zero-copy deserialization

### Configuration Format
- **TOML** (via toml crate)
  - Human-readable configuration
  - Well-suited for settings files
  - Comments support
  - Nested structures
  - *Note: Configuration unchanged for Dioxus*

### State Management
- **Dioxus Hooks & Signals**
  - use_signal for reactive state
  - use_context for global state sharing
  - use_effect for side effects
  - use_coroutine for async operations
  - use_future for async data fetching

- **Backend State (Unchanged)**
  - Arc<RwLock<T>> for IRC engine state
  - HashMap for connection management
  - VecDeque for message buffers

## Scripting & Plugins

### Lua Integration
- **mlua** (v0.9+)
  - Safe Lua bindings
  - Sandboxing capabilities
  - Async function support
  - Good performance
  - Active maintenance

### Python Integration
- **PyO3** (v0.20+)
  - Safe Python bindings
  - GIL management
  - Python 3.8+ support
  - Sandboxing capabilities
  - Excellent performance

### Plugin System
- **libloading** (v0.8+)
  - Dynamic library loading
  - Cross-platform support
  - Safe abstractions

## Development Tools

### CLI Parsing
- **clap** (v4.0+)
  - Declarative argument parsing
  - Automatic help generation
  - Subcommand support
  - Environment variable integration

### Logging
- **tracing** (v0.1)
  - Structured logging
  - Async-aware
  - Multiple subscribers
  - Performance focused

- **tracing-subscriber**
  - Console output formatting
  - File logging
  - Filter configuration

### Error Handling
- **anyhow** (v1.0)
  - Simplified error handling
  - Context adding
  - Backtrace support

- **thiserror** (v1.0)
  - Custom error types
  - Derive macros
  - Standard error trait

## Platform Integration

### System Directories
- **directories** (v5.0)
  - Cross-platform paths
  - XDG compliance on Linux
  - Standard locations on Windows/macOS

### Notifications
- **notify-rust** (v4.0)
  - Cross-platform notifications
  - Native integration
  - Action support

## Testing

### Test Framework
- **Built-in Rust Testing**
  - Unit tests with #[test]
  - Integration tests
  - Documentation tests
  - Benchmark tests

### Mocking
- **mockall** (v0.12)
  - Powerful mocking framework
  - Automocking traits
  - Expectation setting

### Property Testing
- **proptest** (v1.0)
  - Property-based testing
  - Shrinking support
  - Custom strategies

## Build & Distribution

### Build System
- **Dioxus CLI (dx)**
  - `dx serve` for development with hot reload
  - `dx build --platform desktop` for releases
  - `dx fmt` for RSX component formatting
  - WebView bundle management

- **Cargo**
  - Native Rust build system
  - Workspace support  
  - Feature flags
  - Cross-compilation

### CI/CD
- **GitHub Actions**
  - Multi-platform builds
  - Automated testing
  - Release automation
  - Artifact generation

### Packaging
- **cargo-bundle** (optional)
  - Platform packages
  - Installer generation
  - Icon embedding

## Optional Dependencies

### Performance
- **criterion** (v0.5)
  - Statistical benchmarking
  - Regression detection
  - HTML reports

### Security Audit
- **cargo-audit**
  - Vulnerability scanning
  - Advisory database
  - CI integration

### Code Coverage
- **tarpaulin**
  - Line coverage
  - Branch coverage
  - HTML reports

## Version Policy

### Rust Version
- **MSRV**: 1.75.0
  - Stable channel primary
  - Beta channel for testing
  - Feature flags for newer features

### Dependency Updates
- Security updates: Immediate
- Major versions: Careful evaluation
- Minor versions: Regular updates
- Patch versions: Automatic

## Platform Requirements

### Windows
- Windows 10 version 1809+
- Visual C++ Runtime
- WebView2 Runtime (usually pre-installed)
- UTF-8 code page support

### macOS
- macOS 10.15 Catalina+
- Xcode Command Line Tools
- WebKit framework (built-in)
- Homebrew (recommended)

### Linux
- glibc 2.31+ (Ubuntu 20.04+)
- **WebView Dependencies**:
  - webkit2gtk-4.1-devel
  - libsoup3-devel
  - atk-devel
  - gtk3-devel
- X11 or Wayland
- D-Bus for notifications
- PulseAudio/PipeWire for sounds

## Dioxus-Specific Technologies

### Core Dioxus Stack
- **dioxus** (v0.6) - Core framework
- **dioxus-desktop** - Desktop platform target
- **dioxus-router** - Client-side routing
- **dioxus-hooks** - State management hooks
- **dioxus-signals** - Reactive state primitives

### WebView Integration
- **WebKit2GTK** (Linux) - Native WebView
- **WebKit** (macOS) - System framework
- **WebView2** (Windows) - Microsoft Edge WebView

### Development Tools
- **dioxus-cli** - Development server and build tools
- **dioxus-hot-reload** - Live code updates
- **dioxus-devtools** - Component inspection

## Future Considerations

### Dioxus Roadmap
- **Mobile Support** - iOS/Android targets
- **Web Deployment** - WebAssembly builds
- **Server Components** - Full-stack capabilities
- **Native Widgets** - Alternative to WebView

### Potential Additions
- **WebRTC** for voice/video (easier with WebView)
- **SQLite** for message history
- **tantivy** for full-text search
- **Web Components** for rich media embedding

### Experimental Features
- **WASM** plugin support
- **Progressive Web App** features
- **Native mobile** deployment
- **Electron-style** packaging