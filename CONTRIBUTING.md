# Contributing to RustIRC

First off, thank you for considering contributing to RustIRC! It's people like you that will make RustIRC the definitive modern IRC client.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Process](#development-process)
- [Style Guidelines](#style-guidelines)
- [Commit Messages](#commit-messages)
- [Pull Requests](#pull-requests)
- [Community](#community)

## Code of Conduct

This project and everyone participating in it is governed by our Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/doublegate/RustIRC.git
   cd RustIRC
   ```
3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/originalowner/RustIRC.git
   ```
4. **Create a branch** for your work:
   ```bash
   git checkout -b feature/your-feature-name
   ```

### Development Environment Setup - Dioxus Branch

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Dioxus CLI for hot reload development
cargo install dioxus-cli

# Install development tools
rustup component add rustfmt clippy

# Install system dependencies (Linux - Fedora/Bazzite)
rpm-ostree install webkit2gtk4.1-devel libsoup3-devel atk-devel gtk3-devel

# Install system dependencies (Linux - Ubuntu/Debian)
sudo apt install libwebkit2gtk-4.1-dev libsoup-3.0-dev libatk1.0-dev libgtk-3-dev

# Install cargo-watch for development (optional)
cargo install cargo-watch

# Run tests to ensure everything works
cargo test
```

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. When creating a bug report, include:

- **Clear and descriptive title**
- **Exact steps to reproduce**
- **Expected behavior**
- **Actual behavior**
- **Screenshots** (if applicable)
- **System information** (OS, Rust version, etc.)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, include:

- **Clear and descriptive title**
- **Detailed description** of the proposed functionality
- **Rationale** - why would this be useful?
- **Possible implementation** (if you have ideas)

### Code Contributions

1. **Pick an issue** - Look for issues tagged with `good first issue` or `help wanted`
2. **Comment on the issue** to let others know you're working on it
3. **Ask questions** if anything is unclear
4. **Submit a pull request** when ready

### Documentation

- Fix typos and clarify confusing sections
- Add examples to API documentation
- Improve README and guides
- Translate documentation

### Testing

- Add missing tests
- Improve test coverage
- Add integration tests
- Performance benchmarks

## Development Process

### Project Structure - Dioxus Branch

```
RustIRC/
├── crates/
│   ├── rustirc-core/        # Core IRC engine (unchanged)
│   ├── rustirc-protocol/    # Protocol implementation (unchanged)
│   ├── rustirc-dioxus-gui/  # Dioxus GUI implementation
│   ├── rustirc-tui/         # TUI implementation (unchanged)
│   ├── rustirc-scripting/   # Lua/Python scripting (unchanged)
│   └── rustirc-plugins/     # Plugin system (unchanged)
├── src/                     # Main binary entry points
├── Dioxus.toml             # Dioxus configuration
└── assets/                 # Static assets for WebView
```

### Building - Dioxus Branch

```bash
# Development with hot reload (recommended)
dx serve --hot-reload

# Build for desktop
dx build --platform desktop

# Build release version
dx build --release

# Traditional Cargo builds (also work)
cargo build
cargo build --release

# Run specific binary
cargo run --bin dioxus  # Dioxus GUI
cargo run -- --cli      # CLI mode
cargo run -- --tui      # TUI mode

# Format RSX components
dx fmt

# Watch for changes (traditional)
cargo watch -x run
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run benchmarks
cargo bench
```

## Style Guidelines

### Rust Code Style

- **Format**: Always run `cargo fmt` before committing
- **Linting**: Ensure `cargo clippy` passes with no warnings
- **Naming**: Follow Rust naming conventions
  - `snake_case` for functions, variables, modules
  - `CamelCase` for types, traits
  - `SCREAMING_SNAKE_CASE` for constants
- **Documentation**: Document all public APIs
- **Tests**: Write tests for new functionality

Example (Rust code):
```rust
/// Connects to an IRC server with the given configuration.
/// 
/// # Arguments
/// 
/// * `config` - Server configuration
/// 
/// # Returns
/// 
/// Returns `Ok(ConnectionId)` on success, or an error if connection fails.
/// 
/// # Examples
/// 
/// ```
/// let config = ServerConfig::new("irc.libera.chat", 6697);
/// let conn_id = client.connect(config).await?;
/// ```
pub async fn connect(&mut self, config: ServerConfig) -> Result<ConnectionId> {
    // Implementation
}
```

Example (Dioxus component):
```rust
/// IRC Channel component for displaying messages and user interactions.
#[component]
fn ChannelView(cx: Scope, channel_name: String, messages: Vec<IrcMessage>) -> Element {
    let show_users = use_state(cx, || true);
    
    cx.render(rsx! {
        div {
            class: "channel-container",
            MessageList { messages: messages.clone() }
            if *show_users.get() {
                UserList { channel: channel_name.clone() }
            }
        }
    })
}
```

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting, etc.)
- `refactor:` Code refactoring
- `perf:` Performance improvements
- `test:` Test additions or corrections
- `chore:` Maintenance tasks

Examples:
```
feat: add SASL PLAIN authentication support
fix: correctly handle UTF-8 in channel names
docs: update scripting guide with Python examples
```

### Documentation Style

- Use **Markdown** for documentation
- Include **code examples** where helpful
- Keep a **conversational but professional** tone
- Update relevant docs when changing functionality

## Pull Requests

1. **Update your fork**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Ensure quality**:
   - All tests pass
   - Code is formatted (`cargo fmt`)
   - No clippy warnings (`cargo clippy`)
   - Documentation is updated
   - Commit messages follow guidelines

3. **Create the PR**:
   - Use a clear, descriptive title
   - Reference any related issues
   - Describe what changes were made and why
   - Include screenshots for UI changes

4. **PR Review Process**:
   - Maintainers will review your code
   - Address any feedback
   - Once approved, it will be merged

### PR Checklist

- [ ] My code follows the project's style guidelines
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes

## Community

### Communication Channels

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - General discussions and questions
- **IRC** - #rustirc on Libera.Chat (once we're running!)

### Getting Help

- Check the [documentation](docs/)
- Search existing issues
- Ask in GitHub Discussions
- Join us on IRC

### Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Given credit in commit messages

## Development Philosophy

- **User Experience First** - Every feature should improve the user experience
- **Performance Matters** - Keep RustIRC fast and responsive
- **Security by Design** - Consider security implications
- **Cross-Platform** - Ensure features work on all supported platforms
- **Extensible** - Maintain clean APIs for scripts and plugins

## License

By contributing to RustIRC, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).

---

Thank you for contributing to RustIRC! 🦀💬