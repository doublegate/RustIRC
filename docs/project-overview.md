# RustIRC Project Overview - Dioxus v0.6 Branch

## Vision Statement

RustIRC is a modern, cross-platform IRC client that synthesizes the best features from legendary IRC clients while leveraging Rust's safety, performance, and concurrency features. Our goal is to create the definitive IRC client for the modern era - one that is powerful enough for veterans yet accessible to newcomers.

### Dioxus v0.6 Implementation

This branch explores re-implementing RustIRC's user interface using Dioxus v0.6, bringing React-like component development to desktop IRC. This maintains all IRC functionality while modernizing the development experience with:

- **React-like Architecture**: Familiar component patterns for web developers
- **Virtual DOM**: Efficient rendering with smooth UI updates
- **Hot Reload Development**: Instant UI updates during development
- **Modern State Management**: Hooks-based patterns (useState, useEffect, useContext)
- **Rich Content Support**: WebView enables modern web content rendering
- **Future Mobile Ready**: Dioxus supports iOS/Android targets

## The IRC Client Landscape

### Current State
The IRC client ecosystem has reached a critical juncture:
- **HexChat** discontinued in early 2024, leaving a significant void
- **mIRC** remains Windows-only shareware with dated UI
- **WeeChat** excels technically but intimidates with its TUI-only interface
- Many alternatives lack comprehensive features or active maintenance

### The Opportunity
This confluence of factors creates an ideal moment to introduce a new, definitive client that:
- Combines proven features from existing clients
- Leverages modern programming practices
- Ensures long-term sustainability through community involvement
- Provides first-class cross-platform support

## Core Principles

### 1. Best-of-Breed Synthesis
RustIRC doesn't reinvent IRC - it perfects the client experience by combining:

**From mIRC:**
- Powerful event-driven scripting
- Extensive customization capabilities
- Rich context menus and UI flexibility

**From HexChat:**
- Clean, intuitive graphical interface
- Straightforward network configuration
- Multi-language plugin support

**From WeeChat:**
- Lightweight, efficient architecture
- Advanced buffer management
- Cutting-edge protocol support
- Optional terminal interface

### 2. Security by Default
- Rust's memory safety eliminates entire vulnerability classes
- TLS/SSL connections by default
- Sandboxed scripting environment
- Careful input validation and injection prevention

### 3. Performance Excellence
- Handle 100+ channels with minimal resource usage
- Non-blocking UI under all conditions
- Efficient message parsing and rendering
- Leverage Rust's zero-cost abstractions

### 4. Deep Extensibility
- Powerful Lua scripting engine
- Binary plugin architecture
- Built-in script/plugin manager
- Comprehensive event and API system

### 5. True Cross-Platform Experience
- Native look and feel on Windows, macOS, and Linux
- Platform-specific integrations (notifications, file dialogs)
- Consistent functionality across all platforms
- Single codebase, multiple targets

## Feature Highlights

### Core IRC Features
- Multi-server connections
- Full IRC protocol compliance (RFC 1459/2812)
- Complete IRCv3 implementation
- UTF-8 support throughout

### Advanced Protocols
- **DCC Suite**: CHAT, SEND/RECV, RESUME, Passive DCC
- **SASL Authentication**: PLAIN, EXTERNAL, SCRAM-SHA-256
- **IRCv3 Extensions**: Tags, capabilities, history, and more

### User Interface
- Modern GUI with Iced framework
- Optional TUI mode with ratatui
- Flexible window management (tabs, splits, trees)
- Customizable themes and layouts

### Extensibility
- Event-driven Lua scripting
- Binary plugin system
- Script/plugin marketplace
- Custom commands and aliases

### Security & Privacy
- TLS by default with rustls
- Proxy support (SOCKS5, HTTP)
- Secure credential storage
- Optional OTR encryption (via plugin)

## Target Audience

### Casual Users
- Simple setup with network wizards
- Sensible defaults
- Clean, approachable interface
- Built-in help and documentation

### Power Users
- Advanced scripting capabilities
- Extensive customization options
- Keyboard-driven workflows
- Multi-network management

### Developers
- Clean, modular codebase
- Comprehensive documentation
- Plugin development framework
- Active community involvement

## Success Metrics

### Technical Goals
- Zero memory safety issues
- Sub-second startup time
- <50MB memory usage for typical sessions
- 60+ fps UI rendering

### Community Goals
- Active contributor base
- Rich ecosystem of scripts/plugins
- Presence on major IRC networks
- Regular release cadence

### Adoption Goals
- Feature parity with major clients within 6 months
- Cross-platform availability from day one
- Positive user feedback and growth
- Long-term sustainability

## Project Timeline

The project follows a 7-phase development plan over approximately 24-26 weeks:

1. **Research & Setup** (2-4 weeks)
2. **Core Engine** (3-6 weeks)
3. **User Interface** (4-10 weeks)
4. **Scripting & Plugins** (3-6 weeks)
5. **Advanced Features** (4-6 weeks)
6. **Testing & Optimization** (3-6 weeks)
7. **Release & Distribution** (2+ weeks)

## Why Rust?

Rust is chosen not merely for technical reasons but as a strategic decision:

### Technical Benefits
- Memory safety without garbage collection
- Fearless concurrency for network operations
- Modern tooling and package ecosystem
- Cross-platform compilation

### Community Benefits
- Growing, enthusiastic developer community
- Modern language attracts new contributors
- Excellent documentation culture
- Strong emphasis on correctness

### Sustainability Benefits
- Easier onboarding than C/C++
- Compiler prevents entire bug classes
- Refactoring confidence through type system
- Long-term maintainability

## Join Us

RustIRC is an open-source project that welcomes contributions from developers, designers, testers, and IRC enthusiasts. Whether you're interested in core development, plugin creation, documentation, or community building, there's a place for you in the RustIRC project.

Together, we can build the IRC client that the community deserves - one that honors the past while embracing the future.