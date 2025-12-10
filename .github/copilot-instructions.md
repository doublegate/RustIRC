# GitHub Copilot Instructions for RustIRC

## Project Overview

RustIRC is a modern IRC client built in Rust that combines the best features from established IRC clients (mIRC, HexChat, WeeChat). The project prioritizes full IRC protocol compliance (RFC 1459/2812, IRCv3), cross-platform support (Windows, macOS, Linux), and provides multiple interface modes (GUI with Iced 0.13.1, TUI with ratatui, CLI).

**Current Version**: v0.3.8 - Enhanced Iced Material Design GUI  
**Status**: Production-ready with 100% functional Material Design 3 implementation

## Technology Stack

- **Language**: Rust (MSRV: 1.75+)
- **Async Runtime**: Tokio for network I/O
- **GUI Framework**: Iced 0.13.1 with Material Design 3 components
- **TUI Framework**: ratatui for terminal interface
- **Security**: rustls for TLS, complete SASL authentication (PLAIN, EXTERNAL, SCRAM-SHA-256)
- **Scripting**: mlua for Lua integration
- **Architecture**: Event-driven with modular 6-crate workspace structure

## Essential Commands

### Building and Running
```bash
# Build the project
cargo build
cargo build --release

# Run in different modes
cargo run                    # GUI mode (default)
cargo run -- --cli          # CLI prototype mode
cargo run -- --tui          # TUI mode with ratatui
cargo run -- --config path/to/config.toml  # Custom config

# Run tests
cargo test
cargo test -- --nocapture  # Show println! output
```

### Code Quality
```bash
# Format code (required before commits)
cargo fmt

# Lint code (must pass before commits)
cargo clippy -- -D warnings

# Pre-commit check
cargo fmt --check && cargo clippy -- -D warnings

# Generate documentation
cargo doc --open
```

## Coding Standards

### Critical Rules
1. **Zero Placeholder Code**: Never leave "TODO" or "In a real implementation" comments. Implement all functionality completely.
2. **No Removal Strategy**: Fix compilation errors by implementing missing functionality, never by removing/disabling features.
3. **Complete Platform Support**: Implement full Windows (PowerShell), macOS (osascript), Linux (notify-send) support using conditional compilation.
4. **Error Handling**: Always use proper Result types with descriptive error messages. Follow Rust error handling idioms.
5. **Memory Safety**: Leverage Rust's ownership model. Use references appropriately, avoid unnecessary clones unless required for lifetime management.

### Rust Best Practices
- Follow Rust naming conventions (snake_case for functions/variables, CamelCase for types)
- Use `async/await` with Tokio for all network operations
- Implement proper trait bounds and lifetime annotations
- Disable pagers in git commands: `git --no-pager`
- Use `#[cfg(target_os = "...")]` for platform-specific code

### IRC Protocol Specifics
- **Field Naming**: Verify against protocol definitions (e.g., WHOIS uses `targets` not `target`)
- **Message Filtering**: Handle case-sensitive filtering (both "System" and "system")
- **Command Implementation**: Use `rustirc_protocol::Command` for IRC method implementation
- **Protocol Compliance**: Full support for RFC 1459/2812 and IRCv3 extensions

### GUI Development (Iced 0.13.1)
- Use proper border syntax with `0.0.into()` for radius
- Apply container styling for pane dividers
- Implement Material Design 3 patterns with SerializableColor wrapper
- Use `.into()` conversions for automatic color type conversions
- Handle lifetime issues with clone-before-move pattern

## Common Patterns

### Compilation Error Resolution
1. **Type Mismatches**: Convert types properly using Into/From traits
2. **Lifetime Issues**: Use clone-before-move pattern: `let value = data.clone(); move || { use value }`
3. **Borrow Checker**: Extract values before closures to satisfy borrow checker
4. **Platform Code**: Use conditional compilation with complete implementations

### Multi-Server Architecture
- Use `HashMap<String, ServerData>` for server management
- Check server-specific connection state before IRC operations
- Implement proper command routing with server validation
- Provide informative error messages for unavailable servers

### Material Design 3 Integration
- SerializableColor wrapper for config persistence: `[f32; 4]` with serde traits
- Implement `.scale_alpha()` for transparency adjustments
- Use `.build()` APIs for MaterialText/MaterialButton components
- Apply responsive layouts with proper enum traits

## Testing Strategy
- Unit tests for protocol commands
- Integration tests for connection scenarios
- Mock IRC servers for edge cases
- UI tests using framework-specific tools
- Performance benchmarks for message throughput
- Security audits for input validation

## GitHub Actions / CI/CD
- BASH_ENV helper functions required for cross-platform timeout support
- sccache HTTP 400 fallback to local disk cache
- Timeout protection for all cargo operations
- Clippy must run AFTER successful build completion
- All platforms must pass: Windows, Linux (x64/ARM64), macOS (x64/ARM64)

## Directory Structure
```
RustIRC/
├── src/               # Application entry point and main logic
├── crates/            # Modular crate structure (6 crates)
├── tests/             # Integration tests
├── docs/              # User documentation
├── ref_docs/          # Reference materials and development plans
├── to-dos/            # Phase-specific task lists
├── .github/           # GitHub workflows and configurations
└── scripts/           # Development and build scripts
```

## Security Considerations
- All network communication over TLS by default
- Secure credential storage (system keychain integration)
- Sandboxed scripting environment
- Input validation against malformed IRC messages
- DCC security warnings and IP masking options

## Documentation Requirements
- Maintain rustdoc comments for all public APIs
- Include examples in documentation
- Update CHANGELOG.md for all user-facing changes
- Keep README.md badges and status up to date
- Document all architectural decisions

## Performance Goals
- Handle 100+ simultaneous channels without lag
- Efficient user list management with optimized data structures
- Background logging and message processing
- Responsive UI even under heavy message load
- 60 FPS animations in GUI mode

## Common Pitfalls to Avoid
❌ Running clippy before or parallel to build  
❌ Using restore-keys parameter with Swatinem/rust-cache@v2  
❌ Leaving placeholder/stub code  
❌ Removing features to fix compilation errors  
❌ Using matrix.os in shell expressions with workflow_call  
❌ Missing platform-specific implementations  

✅ Implement complete functionality immediately  
✅ Run clippy after successful build  
✅ Use proper error types and handling  
✅ Complete platform support with #[cfg]  
✅ Follow established patterns in codebase  
✅ Update documentation with code changes  

## Getting Help
- Review existing code patterns in the codebase
- Check `docs/` for architectural guidance
- Refer to `ref_docs/` for development plans
- Review `to-dos/` for phase-specific requirements
- Consult IRC protocol specs in `docs/specs/`

When in doubt, prioritize:
1. Memory safety and Rust idioms
2. Complete implementation over partial solutions
3. Cross-platform compatibility
4. Protocol compliance
5. Code quality and documentation
