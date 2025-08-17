# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RustIRC is a modern IRC client being developed in Rust that aims to combine the best features from established IRC clients:

- **mIRC**: Powerful scripting and customization
- **HexChat**: User-friendly GUI and plugin support
- **WeeChat**: Efficiency, scriptability, and buffer management

The project prioritizes full compatibility with IRC standards including IRCv3 extensions, DCC support for file transfers and chats, SASL authentication mechanisms, and cross-platform operation on Linux, macOS, and Windows 10+.

## Development Status

**Documentation Phase Complete** (2025-08-05)

- Comprehensive documentation created in `/docs/`
- Detailed todo lists generated for all 7 phases in `/to-dos/`
- Ready to begin Phase 1: Research & Setup

The repository now contains:

- Development plan documents from various AI assistants in `ref_docs/`
- Complete project documentation in `docs/`
- Phase-specific todo lists in `to-dos/`
- Images for branding (logo and banner)

## Planned Architecture

### Core Components

- **Multi-Server Connection Manager**: Async tasks handling separate connection states per server
- **IRC Protocol Parser**: RFC 1459/2812 compliant with IRCv3 extensions
- **DCC Handler**: Direct Client-to-Client protocol for file transfers and private chats
- **SASL Authentication**: Support for PLAIN, EXTERNAL, and SCRAM-SHA-256 mechanisms
- **Plugin/Script System**: Extensibility through Lua/Python scripts and Rust plugins
- **GUI Framework**: Cross-platform interface using native toolkits or Rust GUI crates
- **TUI Mode**: Terminal-based interface option for efficiency

### Technology Stack (Planned)

- **Language**: Rust
- **Async Runtime**: Tokio for network I/O
- **GUI Options**: iced, egui, or platform-specific bindings
- **TLS**: rustls or native-tls
- **Scripting**: mlua for Lua integration

## Development Commands (Future)

Once the project is set up with Cargo, these will be the common commands:

```bash
# Build the project
cargo build
cargo build --release

# Run tests
cargo test
cargo test -- --nocapture  # Show println! output

# Run the client
cargo run
cargo run -- --config path/to/config.toml

# Code quality
cargo fmt           # Format code
cargo clippy        # Lint code
cargo fmt --check && cargo clippy -- -D warnings  # Pre-commit check

# Documentation
cargo doc --open    # Generate and view documentation

# Cross-platform builds
cargo build --target x86_64-pc-windows-gnu
cargo build --target x86_64-apple-darwin
```

## Key Design Decisions

### IRC Protocol Compliance

- Full support for RFC 1459 and RFC 2812
- Comprehensive IRCv3 implementation including:
  - Capability negotiation (CAP LS/REQ)
  - Message tags and server-time
  - SASL authentication during connection
  - Batch message handling
  - Away notifications and account tracking

### Security Considerations

- All network communication over TLS by default
- Secure credential storage (system keychain integration planned)
- Sandboxed scripting environment to prevent abuse
- Input validation against malformed IRC messages
- DCC security warnings and IP masking options

### Performance Goals

- Handle 100+ simultaneous channels without lag
- Efficient user list management with optimized data structures
- Background logging and message processing
- Responsive UI even under heavy message load

### Extensibility Model

- Event-driven scripting API (on_message, on_join, etc.)
- Plugin system with process/thread isolation
- Theme support via configuration files
- Built-in script/plugin manager for discovery and installation

## Development Workflow

1. **Feature Implementation**: Follow the phased approach outlined in development plans
2. **Testing**: Unit tests for protocol parsing, integration tests for network operations
3. **Cross-Platform**: CI/CD testing on Linux, macOS, and Windows
4. **Documentation**: Maintain user docs and API documentation for plugin developers

## Directory Structure (Planned)

```text
RustIRC/
├── src/
│   ├── main.rs              # Application entry point
│   ├── client/              # Core IRC client logic
│   ├── protocol/            # IRC protocol implementation
│   ├── network/             # Async networking layer
│   ├── ui/                  # GUI and TUI implementations
│   ├── plugins/             # Plugin system
│   └── config/              # Configuration management
├── tests/                   # Integration tests
├── plugins/                 # Built-in plugins
├── scripts/                 # Development and build scripts
├── docs/                    # User documentation
└── ref_docs/                # Reference materials and plans
```

## Important Implementation Notes

- Prioritize memory safety using Rust's ownership model
- Use async/await with Tokio for all network operations
- Implement proper error handling with descriptive messages
- Follow Rust naming conventions and idioms
- Design with modularity to allow easy feature additions
- Consider accessibility in UI design
- Plan for internationalization from the start

## Phase 1 Completion Notes (August 14, 2025)

### Build System Fixes Applied

- Fixed linker configuration (clang → gcc) for Bazzite/Fedora compatibility
- Resolved EventHandler trait dyn compatibility using async_trait
- Added missing dependencies: async-trait, serde_json, toml
- Systematically completed all empty stub files with minimal valid structures

### Current Status

- All 6 crates compile successfully: rustirc-core, rustirc-protocol, rustirc-gui, rustirc-tui, rustirc-scripting, rustirc-plugins
- cargo build, cargo test, cargo run --help, cargo run --tui all functional
- Technology validation prototypes complete
- CI/CD pipeline with GitHub Actions operational
- Ready to begin Phase 2: Core IRC Engine development

## Testing Strategy

- Unit tests for individual protocol commands
- Integration tests for full connection scenarios
- Mock IRC servers for testing edge cases
- UI tests using appropriate frameworks
- Performance benchmarks for message throughput
- Security audits for input validation

## Documentation Structure

### Project Documentation (`/docs/`)

- `README.md` - Documentation index
- `project-overview.md` - Vision and goals
- `architecture-guide.md` - System design
- `technology-stack.md` - Dependencies
- `project-status.md` - Current progress
- `phases/` - Detailed implementation guides for each phase
- `specs/` - Technical specifications (IRC protocol, IRCv3, etc.)

### Development Todos (`/to-dos/`)

- `README.md` - Todo list overview
- `phase1-todos.md` through `phase7-todos.md` - Detailed task lists
- Comprehensive task tracking for all development phases
