# rustirc-scripting

Lua and Python scripting engine for the RustIRC client.

## Overview

The `rustirc-scripting` crate provides a powerful scripting system that allows users to extend and customize the RustIRC client with Lua and Python scripts. It offers:

- **Dual Language Support**: Both Lua and Python scripting
- **Safe Sandboxing**: Secure script execution with resource limits
- **Rich API**: Comprehensive access to IRC client functionality
- **Event-Driven**: Scripts can respond to IRC events and user actions
- **Hot Reloading**: Scripts can be modified and reloaded without restarting
- **Package Management**: Built-in script discovery and installation

*Note: This crate is planned for Phase 4 implementation. Current version provides API definitions and examples.*

## Features

- 🐍 **Python Support** via PyO3 for complex automation
- 🌙 **Lua Support** via mlua for lightweight scripting  
- 🔒 **Sandboxed Execution** with resource and permission controls
- 🎯 **Event System** for responding to IRC activities
- 📦 **Script Management** with automatic loading and organization
- 🔄 **Hot Reloading** for development and debugging
- 📚 **Rich API** for IRC operations and client interaction
- 🛡️ **Security Model** preventing malicious script behavior

## Usage (Planned for Phase 4)

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustirc-scripting = "0.3.3"
features = ["lua", "python"]  # Choose language support
```

### Basic Script Engine Usage

```rust
use rustirc_scripting::{ScriptEngine, ScriptLanguage};
use rustirc_core::events::EventBus;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create event bus for IRC communication
    let event_bus = Arc::new(EventBus::new());
    
    // Initialize script engine
    let mut engine = ScriptEngine::new(event_bus).await?;
    
    // Load and execute a Lua script
    let lua_script = r#"
        function on_message(sender, target, text)
            if text:match("!time") then
                send_message(target, "Current time: " .. os.date())
            end
        end
        
        -- Register event handler
        register_event("message", on_message)
    "#;
    
    engine.load_script("time_bot.lua", lua_script, ScriptLanguage::Lua).await?;
    
    // Load a Python script
    let python_script = r#"
import datetime

def on_join(user, channel):
    if channel == "#welcome":
        send_message(channel, f"Welcome {user}! 👋")
        
def on_command_weather(sender, target, args):
    # This would integrate with weather API
    city = " ".join(args) if args else "Unknown"
    send_message(target, f"Weather in {city}: Sunny, 22°C")

# Register event handlers
register_event("join", on_join)  
register_command("weather", on_command_weather)
    "#;
    
    engine.load_script("welcome_bot.py", python_script, ScriptLanguage::Python).await?;
    
    // Scripts are now active and will respond to events
    Ok(())
}
```

## Script Examples

### Lua Scripts

#### Simple Response Bot
```lua
-- simple_bot.lua
function on_message(sender, target, text)
    -- Respond to greetings
    if text:lower():match("hello") or text:lower():match("hi") then
        send_message(target, "Hello " .. sender .. "! 👋")
    end
    
    -- React to mentions of the bot
    if text:match("rustirc") then
        send_message(target, "You called? I'm here to help!")
    end
end

-- Register the event handler
register_event("message", on_message)

print("Simple bot loaded!")
```

#### Channel Management
```lua
-- channel_manager.lua
local admins = {"alice", "bob", "charlie"}

function is_admin(nick)
    for _, admin in ipairs(admins) do
        if admin == nick then
            return true
        end
    end
    return false
end

function on_command_kick(sender, target, args)
    if not is_admin(sender) then
        send_message(target, sender .. ": Permission denied")
        return
    end
    
    if #args < 1 then
        send_message(target, "Usage: !kick <nick> [reason]")
        return
    end
    
    local nick = args[1]
    local reason = table.concat(args, " ", 2) or "No reason given"
    
    -- Send kick command to IRC server
    send_raw("KICK " .. target .. " " .. nick .. " :" .. reason)
    log("info", sender .. " kicked " .. nick .. " from " .. target)
end

register_command("kick", on_command_kick)
```

### Python Scripts

#### URL Title Fetcher
```python
# url_title.py
import re
import asyncio
import aiohttp
from bs4 import BeautifulSoup

async def fetch_title(url):
    """Fetch the title of a web page"""
    try:
        timeout = aiohttp.ClientTimeout(total=10)
        async with aiohttp.ClientSession(timeout=timeout) as session:
            async with session.get(url) as response:
                if response.content_type.startswith('text/html'):
                    html = await response.text()
                    soup = BeautifulSoup(html, 'html.parser')
                    title = soup.title.string.strip() if soup.title else "No title"
                    return f"Title: {title}"
                else:
                    return f"Content-Type: {response.content_type}"
    except Exception as e:
        return f"Error: {str(e)}"

async def on_message(sender, target, text):
    """Check for URLs in messages and fetch their titles"""
    url_pattern = r'https?://[^\s]+'
    urls = re.findall(url_pattern, text)
    
    for url in urls:
        if len(url) > 10:  # Skip very short URLs
            title = await fetch_title(url)
            send_message(target, f"📄 {title}")

# Register async event handler
register_async_event("message", on_message)

print("URL title fetcher loaded!")
```

#### Statistics Tracker
```python
# stats.py
import json
import os
from datetime import datetime, timedelta
from collections import defaultdict

class StatsTracker:
    def __init__(self):
        self.stats_file = os.path.join(get_data_dir(), "stats.json")
        self.stats = self.load_stats()
        
    def load_stats(self):
        try:
            with open(self.stats_file, 'r') as f:
                return json.load(f)
        except FileNotFoundError:
            return {
                "messages": defaultdict(int),
                "users": defaultdict(lambda: {"messages": 0, "joins": 0}),
                "channels": defaultdict(lambda: {"messages": 0, "users": set()})
            }
    
    def save_stats(self):
        # Convert sets to lists for JSON serialization
        stats_copy = dict(self.stats)
        for channel in stats_copy["channels"]:
            stats_copy["channels"][channel]["users"] = list(stats_copy["channels"][channel]["users"])
        
        with open(self.stats_file, 'w') as f:
            json.dump(stats_copy, f, indent=2)
    
    def record_message(self, sender, channel):
        self.stats["messages"][channel] += 1
        self.stats["users"][sender]["messages"] += 1
        self.stats["channels"][channel]["messages"] += 1
        self.stats["channels"][channel]["users"].add(sender)
        self.save_stats()

# Global stats tracker
tracker = StatsTracker()

def on_message(sender, target, text):
    if target.startswith("#"):  # Only track channel messages
        tracker.record_message(sender, target)

def on_command_stats(sender, target, args):
    if not args:
        # General stats
        total_messages = sum(tracker.stats["messages"].values())
        total_users = len(tracker.stats["users"])
        send_message(target, f"📊 Total: {total_messages} messages from {total_users} users")
    else:
        # Channel-specific stats
        channel = args[0] if args[0].startswith("#") else f"#{args[0]}"
        if channel in tracker.stats["channels"]:
            ch_stats = tracker.stats["channels"][channel]
            msg_count = ch_stats["messages"]
            user_count = len(ch_stats["users"])
            send_message(target, f"📊 {channel}: {msg_count} messages, {user_count} users")
        else:
            send_message(target, f"No stats available for {channel}")

# Register handlers
register_event("message", on_message)
register_command("stats", on_command_stats)

print("Statistics tracker loaded!")
```

## Script API (Phase 4)

### Core Functions

```lua
-- Lua API
send_message(target, text)              -- Send message to channel/user
join_channel(channel, key?)             -- Join IRC channel
part_channel(channel, reason?)          -- Leave IRC channel  
send_raw(command)                       -- Send raw IRC command
log(level, message)                     -- Write to log file
get_channels()                          -- Get list of joined channels
get_users(channel)                      -- Get users in channel
get_config(key, default?)               -- Get configuration value
set_config(key, value)                  -- Set configuration value
```

```python
# Python API
send_message(target: str, text: str) -> None
join_channel(channel: str, key: str = None) -> None  
part_channel(channel: str, reason: str = None) -> None
send_raw(command: str) -> None
log(level: str, message: str) -> None
get_channels() -> List[str]
get_users(channel: str) -> List[str]
get_config(key: str, default=None) -> Any
set_config(key: str, value: Any) -> None
```

### Event Registration

```lua
-- Register event handlers
register_event("message", function(sender, target, text) end)
register_event("join", function(user, channel) end)
register_event("part", function(user, channel, reason) end)
register_event("connect", function(server) end)
register_event("disconnect", function(server, reason) end)

-- Register custom commands
register_command("mycommand", function(sender, target, args) end)
```

```python
# Python event registration
register_event("message", lambda sender, target, text: None)
register_async_event("message", async_message_handler)
register_command("weather", weather_command_handler)
register_timer(60, periodic_task)  # Every 60 seconds
```

### Available Events

- `message`: IRC message received
- `join`: User joined channel
- `part`: User left channel
- `quit`: User quit server
- `nick`: User changed nickname
- `topic`: Channel topic changed
- `mode`: Channel/user mode changed
- `connect`: Connected to server
- `disconnect`: Disconnected from server
- `error`: Error occurred

## Configuration

### Script Engine Settings

```toml
[scripting]
enabled = true
auto_load = true
script_directory = "scripts"
max_memory_mb = 64              # Memory limit per script
max_execution_time_ms = 5000    # Execution timeout
enable_file_access = false     # Allow scripts to access files
enable_network_access = false  # Allow network requests
log_level = "info"

[scripting.lua]
enabled = true
sandbox_level = "strict"        # strict, moderate, permissive
preload_libraries = ["string", "table", "math"]

[scripting.python]
enabled = true  
virtual_env = "scripts/.venv"   # Python virtual environment
allowed_modules = ["json", "datetime", "re", "math"]
pip_install_allowed = false    # Allow scripts to install packages
```

### Script Management

```bash
# Script management commands in IRC
/script load myscript.lua              # Load script
/script unload myscript.lua            # Unload script  
/script reload myscript.lua            # Reload script
/script list                           # List loaded scripts
/script info myscript.lua              # Show script info
/script enable myscript.lua            # Enable script
/script disable myscript.lua           # Disable script
```

## Security Model

### Sandboxing

Scripts run in a restricted environment:

```lua
-- Lua sandbox restrictions
-- Blocked: os.execute, io.popen, loadfile, dofile
-- Limited: io.open (read-only), require (whitelist)
-- Monitored: memory usage, execution time, API calls
```

```python
# Python sandbox restrictions  
# Blocked: __import__, open, exec, eval
# Limited: requests (if network enabled), file access
# Monitored: memory, CPU time, system calls
```

### Permission System

Scripts declare required permissions:

```lua
-- script_permissions.lua
SCRIPT_INFO = {
    name = "File Logger",
    version = "1.0.0",
    permissions = {
        "file_write",  -- Can write to files
        "network",     -- Can make network requests  
        "config"       -- Can modify configuration
    }
}
```

## Development Tools

### Script Debugging

```bash
# Enable script debugging
cargo run -- --script-debug

# View script logs
tail -f ~/.rustirc/logs/scripts.log

# Interactive script console
cargo run -- --script-console
```

### Hot Reloading

Scripts are automatically reloaded when changed:

```toml
[scripting.development]
auto_reload = true              # Watch for file changes
reload_delay_ms = 500          # Debounce reload events  
show_reload_notifications = true
```

## API Documentation

For detailed API documentation (when implemented):

```bash
cargo doc --package rustirc-scripting --open
```

Key modules:
- `engine`: Script engine and execution
- `api`: Script API definitions  
- `sandbox`: Security and sandboxing
- `loader`: Script loading and management

## Dependencies

### Core Dependencies
- **mlua**: Lua scripting engine
- **pyo3**: Python integration
- **tokio**: Async runtime
- **serde**: Configuration serialization

### Optional Dependencies  
- **rusty_v8**: V8 JavaScript engine (planned)
- **rhai**: Embedded Rust scripting (planned)

## Building

```bash
# Build with Lua support
cargo build --package rustirc-scripting --features lua

# Build with Python support  
cargo build --package rustirc-scripting --features python

# Build with all languages
cargo build --package rustirc-scripting --all-features
```

## Examples

See the `examples/` directory for complete script examples:

- `examples/lua/` - Lua script examples
- `examples/python/` - Python script examples  
- `examples/mixed/` - Multi-language script interactions

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../../LICENSE-MIT))

at your option.