# RustIRC Lua Scripting

This directory contains Lua scripts for extending RustIRC functionality. The scripting engine provides a secure, sandboxed environment with comprehensive IRC automation capabilities.

## Table of Contents

- [Overview](#overview)
- [Getting Started](#getting-started)
- [IRC API Reference](#irc-api-reference)
- [Event System](#event-system)
- [Built-in Scripts](#built-in-scripts)
- [Creating Custom Scripts](#creating-custom-scripts)
- [Security & Sandboxing](#security--sandboxing)
- [Best Practices](#best-practices)

## Overview

RustIRC's Lua scripting engine allows you to:

- **Automate IRC tasks**: Auto-away, auto-rejoin, auto-response
- **Enhance functionality**: Custom commands, triggers, notifications
- **Process events**: React to messages, joins, parts, and more
- **Extend the client**: Add features without modifying core code

### Features

- 🔒 **Secure Sandboxing**: Dangerous functions removed (os.execute, file I/O, require)
- 📡 **50+ IRC Functions**: Complete automation capabilities
- 🎯 **Event-Driven**: React to IRC events in real-time
- ⚡ **Fast Execution**: Lua 5.4 with JIT compilation
- 🛠️ **Custom Commands**: Register new commands from scripts
- 📝 **Easy to Learn**: Simple Lua syntax

## Getting Started

###  Loading Scripts

Scripts are automatically loaded from the `scripts/` directory on startup. You can also:

```lua
-- Load a script manually (from RustIRC console)
/script load path/to/script.lua

-- Reload a script
/script reload script_name

-- List loaded scripts
/script list

-- Enable/disable scripts
/script enable script_name
/script disable script_name
```

### Basic Script Structure

```lua
-- my_script.lua

-- Script metadata (optional but recommended)
-- @name: My Custom Script
-- @description: Does something cool
-- @author: Your Name
-- @version: 1.0

-- Initialize your script
local config = {
    enabled = true,
    setting1 = "value"
}

-- Register event handlers
function irc.on_message(event)
    if event.type == "message" then
        local channel = event.params[1]
        local message = event.params[#event.params]

        -- Do something with the message
        if message:find("!hello") then
            irc.privmsg(channel, "Hello there!")
        end
    end
end

-- Register custom commands
irc.commands = irc.commands or {}
irc.commands.mycommand = function(args)
    irc.print("My command executed with args: " .. table.concat(args, ", "))
end

-- Script loaded message
irc.print("My Custom Script loaded!")
```

## IRC API Reference

### Core Functions

#### Connection Management

```lua
-- Send raw IRC message
irc.send(message)

-- Connect to server (if not connected)
irc.connect()

-- Disconnect from server
irc.disconnect()
```

### Messaging Functions

```lua
-- Send private message
irc.privmsg(target, message)
-- Example: irc.privmsg("#channel", "Hello world!")

-- Send notice
irc.notice(target, message)
-- Example: irc.notice("nickname", "Private notice")

-- Send action (/me)
irc.action(target, action)
-- Example: irc.action("#channel", "waves hello")

-- Send CTCP command
irc.ctcp(target, command, [args])
-- Example: irc.ctcp("nickname", "VERSION")

-- Send CTCP reply
irc.ctcp_reply(target, reply)
```

### Channel Operations

```lua
-- Join channel
irc.join(channel, [key])
-- Example: irc.join("#rust")
-- Example: irc.join("#private", "secret123")

-- Leave channel
irc.part(channel, [reason])
-- Example: irc.part("#channel", "Goodbye!")

-- Kick user from channel
irc.kick(channel, user, [reason])
-- Example: irc.kick("#channel", "baduser", "Spam")

-- Get/set channel topic
irc.topic(channel, [topic])
-- Get: irc.topic("#channel")
-- Set: irc.topic("#channel", "New topic here")

-- Set channel mode
irc.mode(channel, mode, [params])
-- Example: irc.mode("#channel", "+o", {"username"})

-- Invite user to channel
irc.invite(user, channel)
-- Example: irc.invite("friend", "#private")
```

### User Functions

```lua
-- Change nickname
irc.nick(new_nick)
-- Example: irc.nick("NewNickname")

-- Query user information
irc.whois(nick)
-- Example: irc.whois("someone")

-- WHO query
irc.who(target)
-- Example: irc.who("#channel")

-- Userhost query
irc.userhost(nicks)
-- Example: irc.userhost({"nick1", "nick2"})

-- Set/unset away status
irc.away([message])
-- Set away: irc.away("Gone for lunch")
-- Unset away: irc.away(nil)

-- Check if users are online
irc.ison(nicks)
-- Example: irc.ison({"friend1", "friend2"})
```

### State Query Functions

```lua
-- Get list of connected servers
local servers = irc.servers()

-- Get list of joined channels
local channels = irc.channels()

-- Get users in a channel
local users = irc.users("#channel")

-- Get current nickname
local nick = irc.my_nick()

-- Check if you have op status in channel
local is_op = irc.is_op("#channel")

-- Check if you have voice in channel
local is_voice = irc.is_voice("#channel")

-- Get current connection ID
local conn_id = irc.connection_id()
```

### UI Functions

```lua
-- Print message to UI
irc.print(message, [target])
-- Example: irc.print("Hello!")
-- Example: irc.print("Channel message", "#channel")

-- Echo to current window
irc.echo(message)

-- Log message with level
irc.log(level, message)
-- Example: irc.log("info", "Script running")
-- Example: irc.log("error", "Something went wrong")

-- Set status bar message
irc.status(message)

-- Send desktop notification
irc.notify(title, message)
-- Example: irc.notify("Important", "You were mentioned!")

-- Play beep sound
irc.beep()
```

## Event System

### Registering Event Handlers

Event handlers are functions that respond to IRC events. Define them with the `irc.on_<event>` naming pattern:

```lua
-- Handle connection events
function irc.on_connected(event)
    irc.print("Connected to: " .. event.connection_id)
end

function irc.on_disconnected(event)
    irc.print("Disconnected: " .. event.reason)
end

-- Handle message events
function irc.on_message(event)
    -- event.connection_id - Server connection
    -- event.command - IRC command (PRIVMSG, NOTICE, etc.)
    -- event.params - Message parameters
    local channel = event.params[1]
    local message = event.params[#event.params]

    irc.log("debug", "Message in " .. channel .. ": " .. message)
end

-- Handle channel events
function irc.on_join(event)
    irc.print("Joined channel: " .. event.channel)
end

function irc.on_part(event)
    irc.print("Left channel: " .. event.channel)
end

-- Handle user events
function irc.on_user_join(event)
    irc.print(event.user .. " joined " .. event.channel)
end

function irc.on_user_part(event)
    irc.print(event.user .. " left " .. event.channel)
end

function irc.on_nick(event)
    irc.print(event.old_nick .. " is now known as " .. event.new_nick)
end

function irc.on_topic(event)
    irc.print("Topic for " .. event.channel .. ": " .. event.topic)
end

-- Handle error events
function irc.on_error(event)
    irc.log("error", "IRC error: " .. event.error)
end
```

### Available Events

| Event Type | Event Data | Description |
|------------|------------|-------------|
| `connected` | `connection_id` | Connected to server |
| `disconnected` | `connection_id`, `reason` | Disconnected from server |
| `message` | `connection_id`, `command`, `params` | IRC message received |
| `join` | `connection_id`, `channel` | Joined a channel |
| `part` | `connection_id`, `channel` | Left a channel |
| `user_join` | `connection_id`, `channel`, `user` | User joined channel |
| `user_part` | `connection_id`, `channel`, `user` | User left channel |
| `nick` | `connection_id`, `old_nick`, `new_nick` | Nickname changed |
| `topic` | `connection_id`, `channel`, `topic` | Topic changed |
| `error` | `connection_id`, `error` | Error occurred |

## Built-in Scripts

### auto_away.lua

Automatically sets away status after idle time.

**Features:**
- Configurable idle threshold (default: 5 minutes)
- Auto-unsets away on activity
- Custom away messages
- `/autoaway <seconds>` command

**Usage:**
```
/autoaway 300    # Set threshold to 5 minutes
/autoaway        # Check current setting
```

### auto_rejoin.lua

Automatically rejoins channels after being kicked.

**Features:**
- Configurable rejoin delay
- Enable/disable functionality
- Prevents rejoin spam

**Usage:**
```
/autorejoin on          # Enable auto-rejoin
/autorejoin off         # Disable auto-rejoin
/autorejoin delay 5     # Set 5 second delay
```

### highlight.lua

Highlights messages containing specified keywords or from specific users.

**Features:**
- Custom keyword list
- User-based highlights
- Desktop notifications
- Sound alerts

**Usage:**
```
/highlight important         # Add keyword
/unhighlight important       # Remove keyword
/highlightuser nickname      # Highlight specific user
```

## Creating Custom Scripts

### Example: Auto-Response Bot

```lua
-- auto_responder.lua
-- Responds to specific triggers

local responses = {
    ["!help"] = "Available commands: !help, !time, !version",
    ["!version"] = "RustIRC v0.3.8 with Lua scripting",
}

function irc.on_message(event)
    if event.type == "message" and #event.params >= 2 then
        local channel = event.params[1]
        local message = event.params[#event.params]

        for trigger, response in pairs(responses) do
            if message:find(trigger, 1, true) then
                irc.privmsg(channel, response)
                break
            end
        end
    end
end

irc.print("Auto-responder loaded with " .. #responses .. " triggers")
```

### Example: Channel Logger

```lua
-- logger.lua
-- Logs channel messages (in memory, respects sandbox)

local log_buffer = {}
local max_lines = 1000

function irc.on_message(event)
    if event.type == "message" then
        local line = os.date("%Y-%m-%d %H:%M:%S") .. " " ..
                     table.concat(event.params, " ")
        table.insert(log_buffer, line)

        -- Keep buffer size limited
        if #log_buffer > max_lines then
            table.remove(log_buffer, 1)
        end
    end
end

-- Custom command to view logs
irc.commands = irc.commands or {}
irc.commands.viewlog = function(args)
    local count = tonumber(args[1]) or 10
    local start = math.max(#log_buffer - count + 1, 1)

    for i = start, #log_buffer do
        irc.echo(log_buffer[i])
    end
end

irc.print("Channel logger active (/viewlog [lines])")
```

### Example: URL Title Fetcher

```lua
-- Note: Network access would require future API expansion
-- This shows the concept
--[[
url_titles.lua
Fetches and displays titles for URLs posted in chat

function irc.on_message(event)
    if event.type == "message" then
        local message = event.params[#event.params]

        -- Match URLs (basic pattern)
        for url in message:gmatch("https?://[%w-_%.%?%.:/%+=&]+") then
            -- This would require http client API
            -- irc.http_get(url, function(response)
            --     local title = response:match("<title>(.-)</title>")
            --     if title then
            --         irc.privmsg(event.params[1], "Title: " .. title)
            --     end
            -- end)
        end
    end
end
]]--
```

## Security & Sandboxing

### What's Removed

For security, the following Lua functions are **removed** or **restricted**:

❌ **File System Access:**
- `io.open`, `io.popen`, `io.tmpfile`
- `io.input`, `io.output`

❌ **Operating System:**
- `os.execute`, `os.exit`
- `os.remove`, `os.rename`, `os.tmpname`

❌ **Module Loading:**
- `require`, `dofile`, `loadfile`

### What's Available

✅ **Safe OS Functions:**
- `os.clock`, `os.date`, `os.difftime`, `os.time`

✅ **Safe Debug:**
- `debug.traceback` (for error reporting)

✅ **Standard Library:**
- `string.*`, `table.*`, `math.*`
- `pairs`, `ipairs`, `next`
- `tonumber`, `tostring`, `type`
- `assert`, `error`, `pcall`, `xpcall`

### Resource Limits

Scripts are subject to:
- **Memory limits**: Prevents excessive memory usage
- **Execution time limits**: Prevents infinite loops
- **API rate limiting**: Prevents spam/flooding

### Best Practices for Security

1. **Validate input**: Always check user input
2. **Handle errors**: Use `pcall` for risky operations
3. **Limit scope**: Use local variables
4. **Avoid sensitive data**: Don't hardcode passwords

```lua
-- Good: Input validation
function irc.on_message(event)
    if event and event.params and #event.params >= 2 then
        local message = event.params[#event.params]
        if type(message) == "string" then
            -- Process message
        end
    end
end

-- Good: Error handling
local success, result = pcall(function()
    -- Risky operation
    return some_function()
end)

if not success then
    irc.log("error", "Operation failed: " .. tostring(result))
end
```

## Best Practices

### Performance

1. **Use local variables**: Faster than globals
2. **Cache frequently used values**: Reduce lookups
3. **Avoid unnecessary work**: Check conditions early
4. **Use appropriate data structures**: Tables are versatile

```lua
-- Good: Local variables and early returns
local function process_message(event)
    if not event or event.type ~= "message" then
        return  -- Early return
    end

    local message = event.params[#event.params]
    local channel = event.params[1]

    -- Use cached locals
    if message:find("!help") then
        irc.privmsg(channel, "Help text here")
    end
end
```

### Code Organization

1. **Group related functions**: Keep code organized
2. **Use descriptive names**: Make code self-documenting
3. **Comment your code**: Explain complex logic
4. **Separate config from code**: Easy to modify

```lua
-- Configuration
local CONFIG = {
    enabled = true,
    idle_timeout = 300,
    away_message = "Auto-away: Idle",
}

-- State
local state = {
    idle_time = 0,
    is_away = false,
}

-- Helper functions
local function reset_idle()
    state.idle_time = 0
end

local function set_away()
    if not state.is_away then
        irc.away(CONFIG.away_message)
        state.is_away = true
    end
end

-- Event handlers
function irc.on_message(event)
    reset_idle()
    -- Handle message
end
```

### Debugging

Use logging to debug scripts:

```lua
-- Debug logging
irc.log("debug", "Variable value: " .. tostring(some_var))

-- Use pcall for error details
local success, error_msg = pcall(function()
    -- Your code
end)

if not success then
    irc.log("error", "Script error: " .. error_msg)
    irc.print("Error occurred, check logs")
end
```

### Testing

Test scripts with various inputs:

```lua
-- Test helper
local function test_function(input, expected)
    local result = your_function(input)
    if result == expected then
        irc.print("✓ Test passed")
    else
        irc.print("✗ Test failed: got " .. tostring(result) ..
                  ", expected " .. tostring(expected))
    end
end

-- Run tests when script loads
test_function("test input", "expected output")
```

## Examples & Templates

### Script Template

```lua
-- script_name.lua
-- @description: What this script does
-- @author: Your Name
-- @version: 1.0.0

-- Configuration
local config = {
    enabled = true,
    -- Add config options
}

-- State
local state = {}

-- Helper Functions
local function helper_function()
    -- Your code
end

-- Event Handlers
function irc.on_message(event)
    if not config.enabled then return end
    -- Handle message
end

-- Custom Commands
irc.commands = irc.commands or {}
irc.commands.mycommand = function(args)
    -- Handle command
end

-- Initialization
irc.print("Script loaded: script_name v1.0.0")
```

## Troubleshooting

### Script Not Loading

1. **Check syntax**: Lua syntax errors prevent loading
2. **Check logs**: Look for error messages
3. **Verify location**: Scripts must be in `scripts/` directory
4. **Check permissions**: Ensure files are readable

### Script Not Working

1. **Add debug logging**: Use `irc.log("debug", "message")`
2. **Check event type**: Verify correct event handler name
3. **Test step by step**: Comment out sections to isolate issues
4. **Verify API calls**: Ensure correct function names and parameters

### Common Errors

```lua
-- Error: attempt to index a nil value
-- Fix: Check if value exists before using
if event and event.params then
    local message = event.params[1]
end

-- Error: attempt to call a nil value
-- Fix: Verify function exists
if irc.privmsg then
    irc.privmsg("#channel", "message")
end

-- Error: stack overflow
-- Fix: Check for infinite recursion/loops
local MAX_ITERATIONS = 1000
local count = 0
while condition and count < MAX_ITERATIONS do
    count = count + 1
    -- Your code
end
```

## Contributing Scripts

Want to share your script? Great!

1. **Document your script**: Add comments and description
2. **Test thoroughly**: Ensure it works in various scenarios
3. **Follow best practices**: Use the template above
4. **Submit a PR**: Share with the community

## Resources

- [Lua 5.4 Reference Manual](https://www.lua.org/manual/5.4/)
- [Lua Users Wiki](http://lua-users.org/wiki/)
- [RustIRC Documentation](../docs/)
- [IRC Protocol Specs](../docs/specs/irc-protocol.md)

## Support

- **Issues**: [GitHub Issues](https://github.com/doublegate/RustIRC/issues)
- **Discussions**: [GitHub Discussions](https://github.com/doublegate/RustIRC/discussions)
- **IRC**: #rustirc on Libera.Chat

---

Happy scripting! 🦀✨
