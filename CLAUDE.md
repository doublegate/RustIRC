# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RustIRC is a modern IRC client being developed in Rust that aims to combine the best features from established IRC clients:

- **mIRC**: Powerful scripting and customization
- **HexChat**: User-friendly GUI and plugin support
- **WeeChat**: Efficiency, scriptability, and buffer management

The project prioritizes full compatibility with IRC standards including IRCv3 extensions, DCC support for file transfers and chats, SASL authentication mechanisms, and cross-platform operation on Linux, macOS, and Windows 10+.

## Development Status

**v0.3.5 Complete GitHub Actions Pipeline Fix** (2025-08-24 3:28 PM EDT)

- **Phase 1**: Research & Setup ✅ (Complete 2025-08-14)
- **Phase 2**: Core IRC Engine ✅ (Complete 2025-08-17) 
- **Phase 3**: User Interface ✅ (Complete 2025-08-17)
- **GUI Fixes**: Comprehensive issue resolution ✅ (Complete 2025-08-21 12:34 AM EDT)
- **CLI Enhancement**: Multi-server architecture with full GUI parity ✅ (Complete 2025-08-21 1:34 AM EDT)
- **Advanced Interface Features**: Tab completion, key handling, command routing ✅ (Complete 2025-08-21 9:18 PM EDT)
- **100% Full Implementation**: All code complete with no stubs or placeholders ✅ (Complete 2025-08-21 10:55 PM EDT)
- **v0.3.5 GitHub Actions Resilience**: Comprehensive sccache HTTP 400 fallback, cross-platform timeout compatibility ✅ (Complete 2025-08-24 1:35 AM EDT)
- **GUI Framework**: Iced 0.13.1 with advanced styling and proper IRC protocol implementation
- **Working Features**: Full IRC client with live server connectivity, professional tab completion, advanced key handling
- **Test Coverage**: Comprehensive test suite with 10+ execute_task scenarios
- **Current Status**: All interface modes operational with resilient CI/CD pipeline, ready for Phase 4
- **Interface Status**: GUI, CLI, and TUI all fully functional with professional-grade user experience

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

### Technology Stack (Implemented)

- **Language**: Rust
- **Async Runtime**: Tokio for network I/O
- **GUI Framework**: Iced 0.13.1 (functional API implementation)
- **TUI Framework**: ratatui
- **TLS**: rustls  
- **Scripting**: mlua for Lua integration
- **Authentication**: Complete SASL implementation (PLAIN, EXTERNAL, SCRAM-SHA-256)
- **Architecture**: Event-driven with modular crate structure

## Development Commands (Current)

The project is fully functional with multiple interface modes:

```bash
# Build the project
cargo build
cargo build --release

# Run tests
cargo test
cargo test -- --nocapture  # Show println! output

# Run the client (multiple modes available)
cargo run                    # GUI mode (simplified Iced 0.13.1 interface)
cargo run -- --cli          # CLI prototype mode for testing
cargo run -- --tui          # TUI mode with ratatui
cargo run -- --config path/to/config.toml  # With custom config

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

## Important Implementation Patterns (August 21, 2025)

### Zero Placeholder Implementation Strategy

**CRITICAL PATTERN**: Never leave placeholder code for future development:

1. **Complete Implementation Required**: Replace all "In a real implementation" comments with working functionality immediately
2. **Platform-Specific Methods**: Implement full Windows (PowerShell), macOS (osascript), Linux (notify-send) support
3. **Network Management**: Real server:port parsing, connection task routing, proper message formats
4. **Dialog Systems**: Complete modal dialogs with app state integration and proper sizing constraints
5. **Error Resolution Priority**: Fix through implementation, never through removal/disabling/stubbing

### Compilation Error Resolution Workflow

1. **Systematic Approach**: Implement everything, never remove/disable functionality to fix errors
2. **Platform Integration**: Use conditional compilation (#[cfg]) with complete implementations
3. **Message Routing**: Ensure proper Task<MessageType> conversions and Into<> implementations
4. **Size Constraints**: Use Iced Size parameters for min/max dialog dimensions
5. **App State Sync**: Preferences dialogs must reflect current application state values

### GUI Debugging & Issue Resolution

When addressing GUI issues in RustIRC:

1. **IRC Protocol Verification**: Always check field names against protocol definitions (e.g., WHOIS uses `targets` not `target/nickmasks`)
2. **Iced 0.13.1 Styling**: Use proper border syntax with `0.0.into()` for radius, container styling for pane dividers
3. **Case-Sensitive Filtering**: Handle both "System" and "system" message senders in filtering logic
4. **State Synchronization**: Use getter methods like `get_filter_state()` to sync UI checkmarks with actual filter states

### Build and Testing Workflow

- Always run `cargo build` to verify fixes before proceeding
- Test GUI functionality with `cargo run` for full interface mode
- Verify all user-reported issues systematically with test cases
- Document all fixes in both code comments and project documentation

### Code Quality Standards

- Zero tolerance for compilation errors - all fixes must result in successful builds
- Zero tolerance for placeholder code - all functionality must be fully implemented
- Maintain backward compatibility while implementing new features
- Follow established patterns in the codebase for consistency
- Update documentation immediately after implementing fixes

### CLI Enhancement Patterns (August 21, 2025)

#### Multi-Server Architecture Migration
When migrating CLI from single-client to multi-server support:

1. **HashMap Storage**: Use `HashMap<String, ServerData>` for scalable server management
2. **State Checking**: Check server-specific connection state before IRC operations
3. **Tab Management**: Implement comprehensive server and channel tab organization
4. **Protocol Commands**: Use `rustirc_protocol::Command` for IRC method implementation
5. **Error Handling**: Server availability checking with informative user messages

#### Interface Mode Parity Achievement
Ensuring CLI has full GUI feature equivalency:

1. **Settings Synchronization**: Theme management, timestamps, compact mode
2. **Command Set Completeness**: All IRC commands matching GUI functionality
3. **Multi-Server Support**: Simultaneous connections with proper organization
4. **Compilation Resolution**: Systematic fixing of architectural migration errors
5. **Foundation Readiness**: Complete interface infrastructure for next phase

### TUI Compilation Error Patterns (August 21, 2025)

**Current TUI Issues Requiring Implementation (Not Removal)**:

1. **Type System Mismatches**: 
   - `handle_key` methods must return `TuiAction` consistently across all input modes
   - Convert `Option<String>` returns to proper `TuiAction` enum variants
   - Fix `Result<TuiAction, Error>` vs `Result<Option<String>, Error>` mismatches

2. **Field Access Issues**:
   - `show_help` field missing from `TuiState` - implement in ui_state structure
   - HashMap key borrowing issues with `&self.current_tab_id.as_ref().unwrap()`

3. **Implementation Strategy**:
   - **Never Remove Features**: Fix type mismatches by implementing proper enum handling
   - **Complete Functionality**: Implement missing fields and state management
   - **Rust Compliance**: Ensure borrow checker satisfaction through proper dereferencing

### GUI Warning Integration Patterns (August 21, 2025)

**Current GUI Warnings Requiring Implementation**:

1. **Dialog System Integration**:
   - Connect `current_font_size`, `current_notifications`, `current_compact` to actual settings UI
   - Implement settings dialog state synchronization

2. **Message Processing Integration**:
   - Connect `irc_message_receiver` to actual IRC message handling pipeline
   - Implement `toggle_user_list` and `update_user_list` functionality

3. **Menu System Integration**:
   - Connect `active_menu` field to menu rendering and state management
   - Implement all menu rendering methods (file, edit, view, server, channel, tools, help)

4. **Testing Framework Integration**:
   - Implement `execute_task` method for test harness functionality
   - Connect test execution to actual GUI testing pipeline

### Advanced Interface Features Complete Pattern (August 21, 2025 9:18 PM EDT)

**Achievement**: Complete interface foundation with professional-grade user experience

1. **Tab Completion System Implementation**:
   - Command completion for IRC commands starting with /
   - Nick completion with proper mention format (nickname: )
   - Channel completion for channels starting with # or &
   - Completion cycling through candidates with Tab key
   - Context-aware completion based on current server and channel

2. **Advanced Key Handling Implementation**:
   - IRC formatting shortcuts (Ctrl+B bold, Ctrl+U underline, Ctrl+I italic)
   - Color codes (Ctrl+K for IRC color codes)
   - History navigation (Ctrl+Up/Down), Message scrolling (PageUp/PageDown)
   - Tab switching (Alt+1-9), Buffer clearing (Ctrl+L)
   - Dialog management (Escape to close dialogs)

3. **Multi-Server Command Routing Implementation**:
   - Enhanced command routing with server validation and proper error handling
   - Command parsing with detailed logging and error recovery
   - Future-ready architecture for true multi-server client connections

4. **Interface Foundation Complete Status**:
   - Zero compilation errors across all interface implementations
   - All interface modes tested for feature parity (GUI, TUI, CLI)
   - Professional-grade user experience matching industry IRC clients
   - Ready for Phase 4 Scripting & Plugins development

### Workflow Compatibility Patterns (August 24, 2025)

**Critical GitHub Actions Fixes Applied**:

1. **Workflow_call Context Issues**:
   - matrix.os not available in shell expressions when called via workflow_call
   - Solution: Remove all matrix.os from shell fields, use bash universally
   - Affects reusable workflows invoked from parent workflows

2. **Script Unification Strategy**:
   - Convert all conditional PowerShell/Bash blocks to unified bash
   - Bash available on all runners including Windows
   - Eliminates 125+ lines of conditional logic

3. **Expression Syntax Requirements**:
   - Use `${{ !contains() }}` not `"!contains()"` for negations
   - Proper GitHub Actions expression syntax prevents parsing errors
   - Apply to all conditional expressions in workflows

4. **Validation Before Push**:
   - Always run `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/file.yml'))"`
   - Catches syntax and context errors before pipeline execution
   - Saves debugging time and failed runs

### GitHub Actions Function Persistence Pattern (August 24, 2025 1:40 AM EDT)

**Critical Pattern for RustIRC Workflow Resilience**:

1. **Function Persistence Issues**:
   - GitHub Actions steps run in separate shell instances
   - Function definitions don't persist between steps
   - Error: `run_with_timeout: command not found` in workflow jobs

2. **BASH_ENV Helper Solution**:
   - Create helper functions in `$RUNNER_TEMP/ci_helpers.sh`
   - Set `BASH_ENV=$RUNNER_TEMP/ci_helpers.sh` in `$GITHUB_ENV`
   - Export functions with `export -f function_name` for universal availability

3. **Cross-Platform Timeout Implementation**:
   - macOS runners lack timeout command (exit code 127)
   - Use perl-based fallback: `perl -e "alarm $duration; exec @ARGV" "$@"`
   - Native timeout for Linux/Windows: `timeout "$duration" "$@"`

4. **Systematic Typo Detection**:
   - Grep analysis identified repeated typos: `run_with_run_with_timeout`
   - Use MultiEdit with `replace_all: true` for comprehensive fixes
   - Applied to both master-pipeline.yml and ci.yml workflows

5. **Comprehensive Doctest Coverage**:
   - Removed Ubuntu-only restrictions: `if: contains(matrix.os, 'ubuntu')`
   - Enabled doctests on all architectures (Linux, macOS, Windows)
   - Ensures consistent validation across all supported platforms

### sccache HTTP 400 Resilience Pattern (August 24, 2025 1:40 AM EDT)

**Critical GitHub Actions Cache Service Outage Handling**:

1. **sccache HTTP 400 Error Pattern**:
   - GitHub Actions cache service returns "Our services aren't available right now"
   - Azure Front Door banner indicates cache service outages
   - Causes build failures with exit code 101 from sccache

2. **Comprehensive Resilience Implementation**:
   - Check sccache availability with `sccache --start-server` probing
   - Automatic fallback to local disk cache on HTTP 400 errors
   - Unset RUSTC_WRAPPER (not just empty) on sccache failure
   - Retry cargo operations without sccache when service unavailable

3. **Local Disk Cache Fallback Configuration**:
   - Set `SCCACHE_GHA_ENABLED=false` to disable GitHub Actions cache
   - Configure `SCCACHE_DIR` and `SCCACHE_CACHE_SIZE` for local storage
   - Provides build continuity during GitHub cache service outages

4. **Unified Workflow Application**:
   - Applied across all 6 test execution steps in both workflows
   - Consistent error handling in master-pipeline.yml and ci.yml
   - Comprehensive timeout protection with cross-platform compatibility

5. **Technical Implementation Details**:
   - Use `if ! sccache --start-server >/dev/null 2>&1; then` for detection
   - Proper variable unsetting with `unset RUSTC_WRAPPER`
   - Timeout protection for all cargo operations using `run_with_timeout`

### Complete GitHub Actions Pipeline Fix Pattern (August 24, 2025 4:41 PM EDT)

**Critical Fixes for v0.3.5 Release Build**:

1. **cargo-nextest Installation Syntax Error Fix**:
   - Problem: Duplicated bash code (lines 247-251) causing unmatched 'fi' statements
   - Solution: Removed redundant code block from ci.yml
   - Impact: Test Matrix jobs now execute successfully on all platforms

2. **MSRV Check Missing Function Fix**:
   - Problem: `run_with_timeout: command not found` error (exit code 127)
   - Solution: Added complete BASH_ENV helper setup to MSRV job
   - Impact: MSRV check now has access to cross-platform timeout functions

3. **Windows Release Build Shell Fix**:
   - Problem: PowerShell attempting to execute bash if-statement syntax
   - Solution: Added `shell: bash` specification to Build release binary step
   - Impact: Windows release artifacts now build successfully

4. **Linux ARM64 Cross-Compilation Fix**:
   - Problem: GLIBC version mismatch (2.33/2.32/2.34/2.39 not found)
   - Solution: Use stable cross v0.2.5 with --locked flag instead of latest git
   - Impact: ARM64 Linux builds now complete successfully

5. **Comprehensive Validation**:
   - All workflow files pass YAML syntax validation
   - All platform builds operational (Windows, Linux x64/ARM64, macOS x64/ARM64)
   - Complete CI/CD pipeline executing without failures
   - All existing resilience features preserved and enhanced

### Workflow Optimization Lessons Learned (August 24, 2025 8:25 PM EDT)

**Critical Anti-patterns Discovered Through v0.3.6 Attempt**:

1. **Build/Clippy Execution Order**:
   - **Anti-pattern**: Running clippy in parallel with or before build
   - **Error**: "can't find crate for iced" (exit code 101)
   - **Solution**: clippy MUST run after successful build completion
   - **Impact**: v0.3.6 pipeline failed, reverted to v0.3.5 stable baseline

2. **Swatinem/rust-cache@v2 Parameters**:
   - **Anti-pattern**: Using restore-keys parameter
   - **Error**: "Unexpected inputs: restore-keys"
   - **Solution**: Only use supported parameters (key, shared-key, save-if)
   - **Documentation**: Action does not support restore-keys despite cache similarity

3. **yamllint Compliance Strategy**:
   - **Preference**: Manual line-by-line fixes over automation
   - **Key fixes**: Document markers (---), truthy values ('on':), line lengths
   - **Result**: 33+ errors fixed while preserving functionality
   - **Learning**: Automation can miss context-sensitive formatting requirements

4. **Workflow Reversion Strategy**:
   - **Approach**: Preserve failed attempts in in_prog/ folder
   - **Documentation**: WORKFLOW_OPTIMIZATION_ATTEMPTS.md with all 14 commits
   - **Benefit**: Future reference for what doesn't work and why
   - **Repository state**: Stable v0.3.5 at commit 4e0fcf6
