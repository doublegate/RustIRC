# RustIRC Scripting and Plugin Guide

## Overview

RustIRC provides a powerful scripting system using Lua 5.4, allowing users to extend functionality without recompiling the client. This guide covers script development, the API reference, security model, and best practices.

## Getting Started

### Basic Script Structure

```lua
-- hello.lua
-- A simple RustIRC script

-- Script metadata
script = {
    name = "Hello World",
    version = "1.0.0",
    author = "Your Name",
    description = "A simple greeting script",
    license = "MIT"
}

-- Event handler for incoming messages
function on_message(server, channel, nick, message)
    -- Respond to !hello command
    if message == "!hello" then
        irc.send_message(server, channel, "Hello, " .. nick .. "!")
    end
end

-- Script initialization
function on_load()
    print("Hello World script loaded!")
    
    -- Register command
    irc.register_command("hello", "Say hello", handle_hello_command)
end

-- Command handler
function handle_hello_command(args)
    local target = args[1] or "world"
    irc.send_message(irc.current_server(), irc.current_target(), "Hello, " .. target .. "!")
end

-- Cleanup on unload
function on_unload()
    print("Hello World script unloaded!")
end
```

### Loading Scripts

Scripts can be loaded through:
1. GUI: Settings → Scripts → Add Script
2. TUI: `/script load <path>`
3. Auto-load: Place in `~/.config/rustirc/scripts/`
4. API: `irc.load_script(path)`

## Core API

### IRC Module

The `irc` module provides core IRC functionality:

```lua
-- Server management
irc.connect(config)                    -- Connect to server
irc.disconnect(server_id)              -- Disconnect from server
irc.current_server()                   -- Get current server ID
irc.servers()                          -- List all server IDs
irc.server_info(server_id)             -- Get server information

-- Messaging
irc.send_message(server, target, text) -- Send PRIVMSG
irc.send_notice(server, target, text)  -- Send NOTICE
irc.send_ctcp(server, target, command, params) -- Send CTCP
irc.send_action(server, target, action) -- Send /me action
irc.send_raw(server, command)          -- Send raw IRC command

-- Channel operations
irc.join(server, channel, key)         -- Join channel
irc.part(server, channel, reason)      -- Leave channel
irc.current_target()                   -- Get current channel/query
irc.channels(server)                   -- List joined channels
irc.channel_info(server, channel)      -- Get channel information

-- User information
irc.current_nick(server)               -- Get our nickname
irc.user_info(server, nick)            -- Get user information
irc.is_user_online(server, nick)       -- Check if user is online
irc.users_in_channel(server, channel)  -- List channel users

-- Modes and topic
irc.set_mode(server, target, modes)    -- Set modes
irc.set_topic(server, channel, topic)  -- Set channel topic
irc.kick(server, channel, nick, reason) -- Kick user
irc.ban(server, channel, mask)         -- Ban user

-- Commands
irc.register_command(name, help, handler) -- Register new command
irc.unregister_command(name)           -- Remove command
irc.execute_command(command)           -- Execute IRC command
```

### Event System

Scripts can handle various IRC events:

```lua
-- Connection events
function on_connect(server)
    -- Called when connected to server
end

function on_disconnect(server, reason)
    -- Called when disconnected
end

-- Message events
function on_message(server, target, nick, message)
    -- PRIVMSG received
end

function on_notice(server, target, nick, message)
    -- NOTICE received
end

function on_action(server, target, nick, action)
    -- /me action received
end

function on_ctcp_request(server, nick, command, params)
    -- CTCP request received
    -- Return response string or nil
end

-- Channel events
function on_join(server, channel, nick, account, realname)
    -- User joined channel
end

function on_part(server, channel, nick, reason)
    -- User left channel
end

function on_quit(server, nick, reason)
    -- User quit IRC
end

function on_kick(server, channel, kicked_nick, kicker_nick, reason)
    -- User was kicked
end

function on_nick_change(server, old_nick, new_nick)
    -- Nick changed
end

function on_topic_change(server, channel, topic, setter)
    -- Topic changed
end

function on_mode_change(server, target, modes, params, setter)
    -- Mode changed
end

-- Raw events
function on_raw_message(server, prefix, command, params)
    -- Raw IRC message
    -- Return true to suppress default handling
end

-- DCC events
function on_dcc_request(server, nick, type, filename, size)
    -- DCC request received
    -- Return true to accept, false to reject
end

function on_dcc_complete(transfer_id)
    -- DCC transfer completed
end

-- Timer events
function on_timer(timer_id)
    -- Timer fired
end
```

### Utility Modules

#### Timer Module

```lua
-- Create timers
timer.once(seconds, callback)          -- One-shot timer
timer.interval(seconds, callback)      -- Repeating timer
timer.cancel(timer_id)                 -- Cancel timer

-- Example: Announce every hour
local announce_timer = timer.interval(3600, function()
    irc.send_message(irc.current_server(), "#channel", "Hourly announcement!")
end)
```

#### HTTP Module

```lua
-- HTTP requests (sandboxed)
http.get(url, callback)                -- GET request
http.post(url, data, callback)         -- POST request

-- Example: Weather command
function handle_weather_command(args)
    local city = table.concat(args, " ")
    local url = "https://api.weather.com/v1/weather?q=" .. http.urlencode(city)
    
    http.get(url, function(response)
        if response.status == 200 then
            local data = json.decode(response.body)
            local temp = data.current.temp_c
            local desc = data.current.condition.text
            irc.send_message(
                irc.current_server(), 
                irc.current_target(),
                string.format("Weather in %s: %d°C, %s", city, temp, desc)
            )
        end
    end)
end
```

#### Storage Module

```lua
-- Persistent storage
storage.set(key, value)                -- Store value
storage.get(key, default)              -- Retrieve value
storage.delete(key)                    -- Delete value
storage.clear()                        -- Clear all data

-- Example: Seen tracker
function on_message(server, target, nick, message)
    storage.set("seen:" .. nick:lower(), {
        time = os.time(),
        channel = target,
        message = message
    })
end

function handle_seen_command(args)
    local nick = args[1]:lower()
    local data = storage.get("seen:" .. nick)
    
    if data then
        local time_ago = os.time() - data.time
        local msg = string.format("%s was last seen %s ago in %s saying: %s",
            args[1], format_duration(time_ago), data.channel, data.message)
        irc.send_message(irc.current_server(), irc.current_target(), msg)
    else
        irc.send_message(irc.current_server(), irc.current_target(), 
            "I haven't seen " .. args[1])
    end
end
```

#### UI Module

```lua
-- User interface interaction
ui.print(text)                         -- Print to current buffer
ui.print_to(buffer, text)              -- Print to specific buffer
ui.create_buffer(name)                 -- Create custom buffer
ui.switch_buffer(buffer)               -- Switch to buffer
ui.prompt(question, callback)          -- Ask user for input
ui.show_notification(title, message)   -- System notification

-- Example: Note taking
local notes_buffer = ui.create_buffer("Notes")

function handle_note_command(args)
    local note = table.concat(args, " ")
    local timestamp = os.date("%Y-%m-%d %H:%M:%S")
    ui.print_to(notes_buffer, timestamp .. " - " .. note)
    storage.set("notes", storage.get("notes", {}) .. "\n" .. timestamp .. " - " .. note)
end
```

## Advanced Features

### Script Configuration

```lua
-- config.lua module for user settings
local config = require("config")

-- Define configuration options
config.define({
    {
        name = "greeting",
        type = "string",
        default = "Hello",
        description = "Greeting message"
    },
    {
        name = "auto_greet",
        type = "boolean", 
        default = true,
        description = "Automatically greet new users"
    },
    {
        name = "greet_delay",
        type = "number",
        default = 5,
        description = "Delay before greeting (seconds)"
    }
})

-- Use configuration
function on_join(server, channel, nick)
    if config.get("auto_greet") and nick ~= irc.current_nick(server) then
        timer.once(config.get("greet_delay"), function()
            local greeting = config.get("greeting")
            irc.send_message(server, channel, greeting .. ", " .. nick .. "!")
        end)
    end
end
```

### Inter-Script Communication

```lua
-- Export functions for other scripts
exports.my_function = function(param)
    return "Result: " .. param
end

-- Import from another script
local other_script = require("other_script")
local result = other_script.my_function("test")

-- Global event system
events.emit("custom_event", data)
events.on("custom_event", function(data)
    -- Handle event
end)
```

### Database Access

```lua
-- SQLite database access
local db = require("database")

-- Open database
local conn = db.open("mydata.db")

-- Create table
conn:execute([[
    CREATE TABLE IF NOT EXISTS karma (
        nick TEXT PRIMARY KEY,
        score INTEGER DEFAULT 0
    )
]])

-- Prepared statements
local stmt = conn:prepare("UPDATE karma SET score = score + ? WHERE nick = ?")
stmt:bind(1, 1)  -- increment
stmt:bind(2, "somenick")
stmt:execute()

-- Queries
local results = conn:query("SELECT * FROM karma ORDER BY score DESC LIMIT 10")
for row in results do
    print(row.nick .. ": " .. row.score)
end
```

### Pattern Matching

```lua
-- Enhanced pattern matching
patterns = require("patterns")

-- IRC-specific patterns
patterns.is_channel(name)              -- Check if valid channel name
patterns.is_nick(name)                 -- Check if valid nickname
patterns.parse_hostmask(mask)          -- Parse nick!user@host
patterns.match_hostmask(mask, host)    -- Check if hostmask matches

-- Example: Admin check
local admins = {
    "*!admin@trusted.host",
    "trusted_nick!*@*"
}

function is_admin(server, nick)
    local info = irc.user_info(server, nick)
    local hostmask = info.nick .. "!" .. info.user .. "@" .. info.host
    
    for _, mask in ipairs(admins) do
        if patterns.match_hostmask(mask, hostmask) then
            return true
        end
    end
    return false
end
```

## Security Model

### Sandboxing

Scripts run in a sandboxed Lua environment with restricted access:

```lua
-- Disabled by default:
-- os.execute()      -- No shell commands
-- io.*              -- No direct file I/O
-- require()         -- Only approved modules
-- load/loadfile     -- No dynamic code loading
-- debug.*           -- No debug library

-- Available safe alternatives:
-- storage.*         -- Persistent data storage
-- http.*            -- Sandboxed HTTP client
-- file.read()       -- Read from scripts directory only
-- file.exists()     -- Check file existence
```

### Permissions

Scripts can request additional permissions:

```lua
-- In script metadata
script = {
    name = "Advanced Script",
    version = "1.0.0",
    permissions = {
        "file_read",      -- Read files from scripts directory
        "file_write",     -- Write files to scripts directory
        "http_request",   -- Make HTTP requests
        "native_module"   -- Load native modules
    }
}
```

### Resource Limits

```lua
-- CPU time limit per event handler
-- Memory usage limit per script
-- Maximum HTTP requests per minute
-- Maximum storage size per script
-- Maximum timer count

-- Scripts exceeding limits are automatically suspended
```

## Plugin Development

### Binary Plugins

For performance-critical or system-integration features, binary plugins can be written in Rust:

```rust
// myplugin/src/lib.rs
use rustirc_plugin::{Plugin, PluginInfo, Context, Result};

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "My Plugin",
            version: "1.0.0",
            author: "Your Name",
            description: "Example binary plugin",
        }
    }
    
    fn on_load(&mut self, ctx: &mut Context) -> Result<()> {
        // Register Lua functions
        ctx.register_function("my_native_function", my_native_function)?;
        
        // Register event handlers
        ctx.on_message(|server, target, nick, message| {
            // Handle message
            Ok(())
        })?;
        
        Ok(())
    }
}

fn my_native_function(ctx: &Context, args: Vec<Value>) -> Result<Value> {
    // Implement native functionality
    Ok(Value::String("Result from Rust".to_string()))
}

// Export plugin
rustirc_plugin::export_plugin!(MyPlugin);
```

### Building Plugins

```toml
# Cargo.toml
[package]
name = "my-plugin"
version = "1.0.0"

[lib]
crate-type = ["cdylib"]

[dependencies]
rustirc-plugin = "1.0"
```

```bash
# Build plugin
cargo build --release

# Install plugin
cp target/release/libmy_plugin.so ~/.config/rustirc/plugins/
```

## Examples

### Auto-Op Bot

```lua
-- auto_op.lua
script = {
    name = "Auto Op",
    version = "1.0.0",
    description = "Automatically op trusted users"
}

local trusted_users = {
    ["#mychannel"] = {
        "trusteduser1",
        "trusteduser2"
    }
}

function on_join(server, channel, nick)
    local trusted = trusted_users[channel:lower()]
    if trusted then
        for _, trusted_nick in ipairs(trusted) do
            if nick:lower() == trusted_nick:lower() then
                -- Delay to avoid flooding
                timer.once(2, function()
                    irc.set_mode(server, channel, "+o " .. nick)
                end)
                break
            end
        end
    end
end
```

### URL Title Fetcher

```lua
-- url_title.lua
script = {
    name = "URL Title",
    version = "1.0.0",
    description = "Fetch and display URL titles",
    permissions = {"http_request"}
}

local url_pattern = "https?://[%w-_.~:/?#@!$&'()*+,;=%%]+"

function on_message(server, target, nick, message)
    -- Find URLs in message
    for url in message:gmatch(url_pattern) do
        fetch_title(server, target, url)
    end
end

function fetch_title(server, target, url)
    http.get(url, function(response)
        if response.status == 200 then
            -- Extract title from HTML
            local title = response.body:match("<title>(.-)</title>")
            if title then
                title = html_decode(title):gsub("\n", " "):gsub("%s+", " ")
                irc.send_message(server, target, "Title: " .. title)
            end
        end
    end)
end

function html_decode(str)
    local entities = {
        ["&amp;"] = "&",
        ["&lt;"] = "<", 
        ["&gt;"] = ">",
        ["&quot;"] = '"',
        ["&#39;"] = "'",
        ["&nbsp;"] = " "
    }
    
    for entity, char in pairs(entities) do
        str = str:gsub(entity, char)
    end
    
    return str
end
```

### Trivia Bot

```lua
-- trivia.lua
script = {
    name = "Trivia Bot",
    version = "1.0.0",
    description = "Channel trivia game"
}

local questions = {
    {
        question = "What year was IRC created?",
        answer = "1988",
        hints = {"19__", "198_"}
    },
    {
        question = "Who created IRC?",
        answer = "Jarkko Oikarinen",
        hints = {"J_____ O_______", "Jarkko O_______"}
    }
    -- Add more questions
}

local game_state = {
    active = false,
    current_question = nil,
    channel = nil,
    server = nil,
    hint_count = 0,
    scores = {}
}

function handle_trivia_command(args)
    if game_state.active then
        irc.send_message(irc.current_server(), irc.current_target(), 
            "A game is already in progress!")
        return
    end
    
    start_game(irc.current_server(), irc.current_target())
end

function start_game(server, channel)
    game_state.active = true
    game_state.server = server
    game_state.channel = channel
    
    irc.send_message(server, channel, "Starting trivia game! Type !stop to end.")
    next_question()
end

function next_question()
    local q = questions[math.random(#questions)]
    game_state.current_question = q
    game_state.hint_count = 0
    
    irc.send_message(game_state.server, game_state.channel, 
        "Question: " .. q.question)
    
    -- Schedule hints
    timer.once(15, give_hint)
    timer.once(30, give_hint)
    timer.once(45, function()
        irc.send_message(game_state.server, game_state.channel,
            "Time's up! The answer was: " .. q.answer)
        timer.once(3, next_question)
    end)
end

function give_hint()
    if not game_state.active then return end
    
    game_state.hint_count = game_state.hint_count + 1
    local hint = game_state.current_question.hints[game_state.hint_count]
    
    if hint then
        irc.send_message(game_state.server, game_state.channel, "Hint: " .. hint)
    end
end

function on_message(server, channel, nick, message)
    if game_state.active and 
       server == game_state.server and 
       channel == game_state.channel then
        
        if message:lower() == game_state.current_question.answer:lower() then
            -- Correct answer!
            game_state.scores[nick] = (game_state.scores[nick] or 0) + 1
            
            irc.send_message(server, channel, 
                string.format("Correct, %s! Your score: %d", 
                    nick, game_state.scores[nick]))
            
            timer.once(3, next_question)
        end
    end
end

-- Register command
irc.register_command("trivia", "Start trivia game", handle_trivia_command)
```

## Best Practices

### Performance

1. **Minimize event handler work** - Use timers for delayed operations
2. **Cache expensive operations** - Store results in variables
3. **Batch operations** - Group multiple messages/commands
4. **Use prepared statements** - For database queries
5. **Limit timer usage** - Cancel unused timers

### Error Handling

```lua
-- Always use pcall for error-prone operations
local success, result = pcall(function()
    -- Potentially failing code
    return json.decode(data)
end)

if success then
    -- Use result
else
    -- Handle error
    print("Error: " .. result)
end

-- Custom error handler
function safe_handler(handler)
    return function(...)
        local success, err = pcall(handler, ...)
        if not success then
            print("Script error: " .. err)
        end
    end
end

-- Wrap handlers
on_message = safe_handler(on_message)
```

### Debugging

```lua
-- Debug utilities
debug.print(...)                       -- Print to debug console
debug.inspect(value)                   -- Pretty-print tables
debug.trace()                          -- Print stack trace

-- Logging
log.debug("Debug message")
log.info("Info message")
log.warn("Warning message")
log.error("Error message")

-- Development helpers
if script.debug then
    debug.print("Message received:", server, target, nick, message)
end
```

### Script Distribution

```lua
-- Package multiple files
-- myproject/
--   init.lua      (main entry point)
--   config.lua    (configuration)
--   commands.lua  (command handlers)
--   package.json  (metadata)

-- package.json
{
    "name": "my-script-pack",
    "version": "1.0.0",
    "main": "init.lua",
    "author": "Your Name",
    "description": "Script collection",
    "dependencies": {
        "json": "^1.0",
        "http": "^1.0"
    }
}
```

## Script Repository

Scripts can be shared through the community repository:
- Browse: https://scripts.rustirc.org
- Submit: `irc script publish`
- Install: `irc script install <name>`
- Update: `irc script update [name]`

## Troubleshooting

### Common Issues

1. **Script not loading**
   - Check syntax: `lua -l init.lua`
   - Verify permissions in metadata
   - Check RustIRC logs

2. **Performance problems**
   - Profile with `debug.profile()`
   - Reduce timer frequency
   - Cache expensive operations

3. **Memory leaks**
   - Clear unused variables
   - Cancel timers on unload
   - Limit stored data size

### Getting Help

- Documentation: https://docs.rustirc.org/scripting
- Examples: https://github.com/rustirc/scripts
- Community: #rustirc on Libera.Chat
- Issues: https://github.com/rustirc/rustirc/issues