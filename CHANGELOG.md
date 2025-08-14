# Changelog

All notable changes to RustIRC will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Phase 1 Completed (2025-08-14)

#### Added
- Initial Cargo workspace structure with 6 crates
- Comprehensive documentation structure
- Architecture Decision Records (ADRs 001-005)
- Technology validation prototypes:
  - GUI prototype using Iced (handles 10k messages)
  - TUI prototype using Ratatui (vi-like controls)
  - Network layer with Tokio (async IRC parsing)
  - Lua scripting with mlua (sandboxed execution)
- Core crate implementations:
  - rustirc-core: Client management, events, state
  - rustirc-protocol: Message parsing, IRCv3 caps
  - rustirc-gui: Iced application structure
  - rustirc-tui: Ratatui application structure
  - rustirc-scripting: Lua engine foundation
  - rustirc-plugins: Plugin manager foundation
- CI/CD pipeline with GitHub Actions
- Development environment configuration
- IRC client analysis report (mIRC, HexChat, WeeChat)

#### Infrastructure
- Git repository initialized and pushed to GitHub
- MIT license added
- rustfmt and clippy configuration
- Criterion benchmarking setup
- VS Code workspace settings
- EditorConfig for consistent formatting
- GitHub Actions for CI/CD

#### Documentation
- ARCHITECTURE.md with system design
- CONTRIBUTING.md with guidelines
- Getting Started development guide
- 5 Architecture Decision Records
- IRC client analysis research
- Phase-specific todo lists (249 tasks)

## [0.1.0] - TBD (Phase 1 Completion)

_This section will be updated when Phase 1 is complete_

### Planned
- Development environment setup
- Technology validation prototypes
- GUI framework comparison (Iced vs GTK-rs)
- Core architecture implementation
- Basic project infrastructure

---

## Release Planning

### Version 0.1.0 - Foundation (Phase 1-2)
- Core architecture
- Basic IRC protocol
- Development infrastructure

### Version 0.2.0 - Interface (Phase 3)
- GUI implementation
- TUI implementation
- Theme system

### Version 0.3.0 - Extensibility (Phase 4)
- Lua scripting
- Python scripting
- Plugin system

### Version 0.4.0 - Advanced Features (Phase 5)
- DCC support
- Full IRCv3
- Security features

### Version 0.5.0 - Beta (Phase 6)
- Performance optimization
- Comprehensive testing
- Beta program

### Version 1.0.0 - Release (Phase 7)
- First stable release
- Cross-platform packages
- Full documentation