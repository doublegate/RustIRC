# rustirc-tui

Terminal-based user interface for the RustIRC client using ratatui.

## Overview

The `rustirc-tui` crate provides a powerful terminal-based interface for the RustIRC client built with ratatui. It offers a full-featured IRC experience in the terminal with:

- **Rich Terminal Interface**: Full-featured TUI with multiple panes and views
- **Vim-like Navigation**: Efficient keyboard navigation and shortcuts
- **Multi-Server Support**: Connect to multiple IRC servers simultaneously
- **Customizable Layout**: Configurable panes, colors, and key bindings
- **Cross-Terminal**: Works across different terminal emulators
- **Low Resource Usage**: Minimal memory and CPU footprint

## Features

- 🖥️ **Full Terminal Interface** with panes, tabs, and status bars
- ⌨️ **Vim-inspired Keybindings** for efficient navigation
- 🎨 **Customizable Themes** with 256-color and truecolor support
- 📑 **Tab Management** for multiple servers and channels
- 🔍 **Built-in Search** with highlighting and filtering
- 📜 **Scrollback Buffer** with efficient memory management
- 🚀 **Fast Performance** optimized for terminal environments
- 🔧 **Configurable Interface** with customizable layouts

## Usage

Run the TUI interface:

```bash
# Start TUI mode
cargo run -- --tui

# Start with specific configuration
cargo run -- --tui --config config.toml

# Start in debug mode
cargo run -- --tui --debug

# Connect to specific server on startup
cargo run -- --tui --connect irc.libera.chat:6697
```

### Embedding the TUI

```rust
use rustirc_tui::app::TuiApp;
use rustirc_tui::config::TuiConfig;
use rustirc_core::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize terminal
    let terminal = rustirc_tui::terminal::init()?;
    
    // Create TUI configuration
    let config = TuiConfig::default();
    
    // Create application state
    let app_state = AppState::default();
    
    // Run TUI application
    let mut app = TuiApp::new(config, app_state);
    app.run(terminal).await?;
    
    Ok(())
}
```

## Interface Layout

### Main TUI Layout

```
┌─ RustIRC ─────────────────────────────────────────────────┐
│ Server: irc.libera.chat | Mode: Normal | Time: 12:34:56  │
├─────────────────┬─────────────────────┬───────────────────┤
│ Servers/Channels│ Messages            │ Users             │
│                 │                     │                   │
│ ● libera.chat   │ 12:34 <alice> Hello │ @op_user          │
│   ├ #rust       │ 12:35 <bob> Hi      │ +voice_user       │
│   ├ #programming│ 12:36 * alice waves │  regular_user     │
│   └ query:carol │ 12:37 <me> Hey all  │  away_user        │
│                 │                     │                   │
│ ○ oftc.net      │                     │                   │
│   └ #debian     │                     │                   │
├─────────────────┴─────────────────────┴───────────────────┤
│ [#rust] Type message... | INS | Lag: 42ms               │
└─────────────────────────────────────────────────────────┘
```

### Key Interface Elements

- **Header Bar**: Connection info, mode, and time
- **Server List**: Tree view of servers and channels
- **Message Area**: Chat messages with formatting
- **User List**: Channel members with status
- **Input Line**: Command and message input
- **Status Line**: Current state and connection info

## Navigation and Keys

### Main Mode Keys

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch between panes |
| `j` / `k` | Navigate up/down in lists |
| `h` / `l` | Navigate left/right in tabs |
| `Enter` | Select/activate item |
| `Esc` | Return to main mode |
| `q` | Quit application |
| `:` | Command mode |
| `/` | Search mode |
| `i` | Insert mode (input) |

### Server/Channel Navigation

| Key | Action |
|-----|--------|
| `Enter` | Join/switch to channel |
| `d` | Disconnect from server |
| `r` | Reconnect to server |
| `n` | New server connection |
| `c` | Create new channel tab |
| `x` | Close current tab |

### Message Navigation

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll messages up/down |
| `G` | Go to bottom of messages |
| `gg` | Go to top of messages |
| `Ctrl+U` / `Ctrl+D` | Page up/down |
| `/` | Search in messages |
| `n` / `N` | Next/previous search result |

### Input Mode Keys

| Key | Action |
|-----|--------|
| `Esc` | Exit input mode |
| `Enter` | Send message/command |
| `Tab` | Autocomplete |
| `Ctrl+A` | Beginning of line |
| `Ctrl+E` | End of line |
| `Ctrl+W` | Delete word backward |
| `Ctrl+U` | Delete line |

## Commands

The TUI supports IRC commands and TUI-specific commands:

### IRC Commands
```
/connect irc.libera.chat:6697
/join #rust
/part #rust Goodbye!
/msg nick Hello there
/nick newnickname
/quit Goodbye!
```

### TUI Commands
```
:config reload          " Reload configuration
:theme <name>           " Change theme
:layout <name>          " Change layout
:log <level>            " Set log level
:help                   " Show help
:quit                   " Quit application
```

### Search Commands
```
/text                   " Search for 'text' in messages  
/text/i                 " Case-insensitive search
/text/r                 " Regex search
n                       " Next search result
N                       " Previous search result
```

## Configuration

### TUI Configuration File

```toml
[tui]
# Interface settings
layout = "three_pane"           # three_pane, two_pane, single_pane
theme = "default"               # Theme name
show_timestamps = true          # Show message timestamps
compact_mode = false            # Compact message display
enable_mouse = true             # Mouse support
vim_mode = true                 # Vim-like navigation

[tui.colors]
# Custom color scheme
background = "black"
foreground = "white"
highlight = "yellow"
accent = "blue"
error = "red"
warning = "yellow"
info = "cyan"
success = "green"

[tui.keybindings]
# Custom key bindings
quit = "q"
command_mode = ":"
search_mode = "/"
insert_mode = "i"
next_tab = "l"
prev_tab = "h"
scroll_up = "k"
scroll_down = "j"

[tui.layout]
# Layout configuration
server_list_width = 20          # Server pane width (%)
user_list_width = 15            # User pane width (%)
message_buffer_size = 1000      # Messages to keep in memory
input_history_size = 100        # Input history entries
```

### Theme System

Built-in themes:
- `default`: Standard terminal colors
- `dark`: Dark color scheme
- `light`: Light color scheme  
- `nord`: Nord color palette
- `dracula`: Dracula theme
- `gruvbox`: Gruvbox colors
- `solarized`: Solarized theme

Custom theme example:
```toml
[themes.custom]
name = "My Theme"
background = "#1a1a1a"
foreground = "#ffffff"
accent = "#00d4aa"
highlight = "#ffd700"
error = "#ff6b6b"
warning = "#feca57"
info = "#48cae4"
success = "#06ffa5"
```

## Message Formatting

The TUI supports rich text formatting within terminal constraints:

### IRC Colors and Formatting
- **Bold**: `^B` codes displayed with bright colors
- **Italic**: `^I` codes displayed with dim/italic where supported
- **Underline**: `^U` codes displayed with underline
- **Colors**: IRC color codes mapped to terminal colors

### Special Highlighting
- **Mentions**: Your nickname highlighted in different color
- **URLs**: Hyperlinks detected and highlighted
- **Channel Links**: Channel names highlighted and clickable
- **Timestamps**: Consistent time format display

### Unicode Support
Full Unicode support including:
- Emoji rendering
- International characters
- Box drawing characters for UI
- Powerline symbols for status bar

## Performance

Optimized for terminal environments:

- **Low Memory**: <10MB typical usage
- **Fast Rendering**: 60fps smooth scrolling
- **Efficient Updates**: Only redraws changed areas
- **Background Processing**: Non-blocking message handling
- **Configurable Buffers**: Adjustable history sizes

### Performance Tuning
```toml
[tui.performance]
max_messages = 1000        # Messages per channel
refresh_rate = 60          # FPS limit
enable_smooth_scroll = true
lazy_render = true         # Only render visible content
background_updates = true  # Process updates in background
```

## Terminal Compatibility

### Tested Terminals
- **xterm**: Full support
- **gnome-terminal**: Full support
- **konsole**: Full support
- **iTerm2**: Full support with enhanced features
- **Windows Terminal**: Full support
- **tmux/screen**: Full support within multiplexers

### Required Features
- **Minimum**: 80x24 characters, 16 colors
- **Recommended**: 256 colors, mouse support
- **Enhanced**: Truecolor (24-bit), Unicode support

### Terminal Capabilities Detection
```rust
use rustirc_tui::terminal::capabilities;

let caps = capabilities::detect();
println!("Colors: {}", caps.color_count);
println!("Mouse: {}", caps.mouse_support);
println!("Unicode: {}", caps.unicode_support);
```

## Accessibility

- **Screen Reader**: Basic support via terminal accessibility
- **High Contrast**: Color schemes designed for visibility
- **Large Text**: Configurable with terminal font scaling
- **Keyboard Only**: Full functionality without mouse
- **Simple Navigation**: Consistent key bindings

## Testing

```bash
# Run TUI tests
cargo test --package rustirc-tui

# Test in different terminal sizes
TERM_COLUMNS=80 TERM_LINES=24 cargo test --package rustirc-tui

# Test with specific terminal type
TERM=xterm-256color cargo test --package rustirc-tui

# Interactive testing
cargo run --package rustirc-tui --example demo
```

## API Documentation

For detailed API documentation:

```bash
cargo doc --package rustirc-tui --open
```

Key modules:
- `app`: Main TUI application logic
- `ui`: User interface components and rendering
- `input`: Input handling and key bindings
- `themes`: Theme system and color management
- `terminal`: Terminal capabilities and setup

## Dependencies

- **ratatui**: Terminal UI framework
- **crossterm**: Cross-platform terminal control
- **tokio**: Async runtime
- **unicode-width**: Text width calculation
- **regex**: Search and highlighting

## Building

```bash
# Standard build
cargo build --package rustirc-tui

# Release build
cargo build --package rustirc-tui --release

# With all features
cargo build --package rustirc-tui --all-features
```

## Troubleshooting

### Common Issues

**Colors not working**:
```bash
# Check terminal color support
echo $TERM
tput colors
```

**Unicode issues**:
```bash
# Set locale
export LC_ALL=en_US.UTF-8
export LANG=en_US.UTF-8
```

**Key bindings not working**:
```bash
# Test terminal input
showkey -a
```

**Performance issues**:
```toml
[tui.performance]
max_messages = 500      # Reduce buffer size
refresh_rate = 30       # Lower FPS
lazy_render = true      # Enable lazy rendering
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../../LICENSE-MIT))

at your option.