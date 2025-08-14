# Getting Started with RustIRC Development

## Prerequisites

- Rust 1.75.0 or later
- Git
- C compiler (for some dependencies)
- pkg-config (Linux/macOS)
- OpenSSL development headers (Linux)

## Setting Up Your Development Environment

### 1. Clone the Repository

```bash
git clone https://github.com/doublegate/RustIRC.git
cd RustIRC
```

### 2. Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 3. Install Development Tools

```bash
# Required components
rustup component add rustfmt clippy rust-analyzer

# Optional but recommended
cargo install cargo-watch   # Auto-rebuild on changes
cargo install cargo-edit    # Manage dependencies
cargo install cargo-audit   # Security audits
cargo install cargo-outdated # Check for outdated deps
```

### 4. Platform-Specific Setup

#### Linux
```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# Fedora/RHEL
sudo dnf install gcc pkg-config openssl-devel

# Arch
sudo pacman -S base-devel pkg-config openssl
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# If using Homebrew
brew install pkg-config openssl
```

#### Windows
- Install Visual Studio 2022 with C++ build tools
- Or install MinGW-w64 for GNU toolchain

## Building RustIRC

### Development Build
```bash
cargo build
```

### Release Build
```bash
cargo build --release
```

### Run Tests
```bash
cargo test
```

### Run with Logging
```bash
RUST_LOG=debug cargo run
```

## Project Structure

```
RustIRC/
├── crates/                 # Workspace crates
│   ├── rustirc-core/      # Core client functionality
│   ├── rustirc-protocol/  # IRC protocol implementation
│   ├── rustirc-gui/       # GUI using Iced
│   ├── rustirc-tui/       # TUI using Ratatui
│   ├── rustirc-scripting/ # Lua scripting engine
│   └── rustirc-plugins/   # Plugin system
├── docs/                   # Documentation
│   ├── adr/               # Architecture decisions
│   ├── development/       # Development guides
│   └── research/          # Research documents
├── prototypes/            # Technology prototypes
└── to-dos/               # Phase-specific todo lists
```

## Common Development Tasks

### Running in Development Mode
```bash
# GUI mode (default)
cargo run

# TUI mode
cargo run -- --tui

# Connect to specific server
cargo run -- --server irc.libera.chat --port 6697
```

### Code Quality Checks
```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Both checks (do before committing)
cargo fmt --check && cargo clippy -- -D warnings
```

### Working with Workspaces
```bash
# Build specific crate
cargo build -p rustirc-protocol

# Test specific crate
cargo test -p rustirc-core

# Run benchmarks
cargo bench
```

### Debugging

#### Using VS Code
1. Install rust-analyzer extension
2. Use provided launch configurations
3. Set breakpoints and press F5

#### Using Command Line
```bash
# Build with debug symbols
cargo build

# Run with LLDB
lldb target/debug/rustirc

# Run with GDB
gdb target/debug/rustirc
```

#### Debug Logging
```bash
# Set log level
RUST_LOG=trace cargo run

# Log specific modules
RUST_LOG=rustirc_core=debug,rustirc_protocol=trace cargo run

# Log to file
RUST_LOG=debug cargo run 2> debug.log
```

## Development Workflow

### 1. Create a Feature Branch
```bash
git checkout -b feature/your-feature
```

### 2. Make Your Changes
- Follow existing code patterns
- Add tests for new functionality
- Update documentation

### 3. Test Your Changes
```bash
# Run tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### 4. Check Code Quality
```bash
# Format
cargo fmt

# Lint
cargo clippy

# Audit dependencies
cargo audit
```

### 5. Commit Your Changes
```bash
git add .
git commit -m "feat: add your feature"
```

### 6. Push and Create PR
```bash
git push origin feature/your-feature
# Then create PR on GitHub
```

## Tips and Tricks

### Fast Iteration
```bash
# Auto-rebuild on changes
cargo watch -x run

# Auto-test on changes
cargo watch -x test

# Check without building
cargo check
```

### Performance Profiling
```bash
# Build with profiling
cargo build --release

# Run with perf (Linux)
perf record --call-graph=dwarf target/release/rustirc
perf report

# Run with Instruments (macOS)
cargo instruments -t "Time Profiler"
```

### Documentation
```bash
# Build and open docs
cargo doc --open

# Include private items
cargo doc --document-private-items --open
```

## Troubleshooting

### Common Issues

#### Compilation Errors
- Ensure Rust version is 1.75.0+: `rustup update`
- Clean build: `cargo clean && cargo build`
- Update dependencies: `cargo update`

#### TLS Issues
- Linux: Install OpenSSL dev packages
- macOS: `export PKG_CONFIG_PATH="/usr/local/opt/openssl/lib/pkgconfig"`
- Windows: Use `rustls` feature instead

#### Performance Issues
- Use release build: `cargo build --release`
- Enable LTO: See Cargo.toml `[profile.release]`
- Profile to find bottlenecks

## Getting Help

- Check documentation in `/docs`
- Look at examples in `/prototypes`
- Ask in GitHub Discussions
- IRC: #rustirc on Libera.Chat (coming soon!)

## Next Steps

1. Read the [Architecture Guide](../ARCHITECTURE.md)
2. Review [Architecture Decision Records](../adr/)
3. Check [Phase 1 TODOs](../../to-dos/phase1-todos.md)
4. Start with a small issue labeled "good first issue"