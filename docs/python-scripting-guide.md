# RustIRC Python Scripting Guide

## Overview

RustIRC provides comprehensive Python scripting support alongside Lua, allowing developers to extend the IRC client using Python 3.8+. This guide covers Python script development, API reference, security considerations, and integration with the RustIRC ecosystem.

## Getting Started with Python Scripts

### Basic Script Structure

```python
#!/usr/bin/env python3
"""
hello_world.py - A simple RustIRC Python script
"""

# Script metadata (required)
__name__ = "Hello World"
__version__ = "1.0.0"
__author__ = "Your Name"
__description__ = "A simple greeting bot"
__license__ = "MIT"

import irc
from datetime import datetime

# Global configuration
config = {
    "greeting": "Hello",
    "channels": ["#general", "#python"],
    "enabled": True
}

def on_load():
    """Called when the script is loaded"""
    print(f"[{__name__}] Script loaded at {datetime.now()}")
    
    # Register commands
    irc.register_command("hello", cmd_hello, "Send a greeting")
    irc.register_command("config", cmd_config, "Manage configuration")
    
    # Join configured channels
    server = irc.current_server()
    for channel in config["channels"]:
        irc.join(server, channel)

def on_unload():
    """Called when the script is unloaded"""
    print(f"[{__name__}] Script unloaded")
    # Cleanup resources if needed

def on_message(server_id, target, nick, message):
    """Handle incoming messages"""
    if not config["enabled"]:
        return
    
    # Respond to mentions
    my_nick = irc.current_nick(server_id)
    if my_nick.lower() in message.lower():
        response = f"{config['greeting']}, {nick}!"
        irc.send_message(server_id, target, response)

def cmd_hello(args):
    """!hello command handler"""
    target = args[0] if args else "World"
    server = irc.current_server()
    channel = irc.current_target()
    
    message = f"{config['greeting']}, {target}!"
    irc.send_message(server, channel, message)
    
def cmd_config(args):
    """!config command handler"""
    if len(args) < 2:
        return "Usage: !config <get|set> <key> [value]"
    
    action = args[0].lower()
    key = args[1]
    
    if action == "get":
        value = config.get(key, "Not set")
        return f"{key} = {value}"
    elif action == "set" and len(args) > 2:
        value = " ".join(args[2:])
        config[key] = value
        return f"Set {key} = {value}"
```

### Loading Python Scripts

1. **Manual Loading**: `/script load /path/to/script.py`
2. **Auto-loading**: Place scripts in `~/.config/rustirc/scripts/python/`
3. **GUI**: Settings → Scripts → Add Python Script
4. **From Script**: `irc.load_script("script.py")`

## Python IRC API Reference

### Core Module: `irc`

The `irc` module is automatically available to all scripts and provides the main interface to RustIRC functionality.

#### Connection Management

```python
# Server operations
server_id = irc.connect({
    "address": "irc.libera.chat",
    "port": 6697,
    "use_tls": True,
    "nick": "MyBot",
    "password": "optional_pass"
})

irc.disconnect(server_id, "Goodbye!")
irc.reconnect(server_id)

# Query server information
servers = irc.servers()  # List of server IDs
info = irc.server_info(server_id)  # Server details
is_connected = irc.is_connected(server_id)
```

#### Messaging Functions

```python
# Send messages
irc.send_message(server_id, target, "Hello, world!")
irc.send_notice(server_id, target, "Notice text")
irc.send_action(server_id, target, "waves")
irc.send_ctcp(server_id, target, "VERSION")
irc.send_raw(server_id, "WHOIS nickname")

# Message formatting
formatted = irc.format_text("Hello", color="red", bold=True)
stripped = irc.strip_formatting(message)
```

#### Channel Operations

```python
# Channel management
irc.join(server_id, "#channel")
irc.join(server_id, "#private", "channelkey")
irc.part(server_id, "#channel", "Leaving...")

# Channel information
channels = irc.channels(server_id)
info = irc.channel_info(server_id, "#channel")
users = irc.channel_users(server_id, "#channel")
topic = irc.channel_topic(server_id, "#channel")

# Channel modes
irc.set_mode(server_id, "#channel", "+m")
irc.set_topic(server_id, "#channel", "New topic")
irc.kick(server_id, "#channel", "nick", "Reason")
irc.ban(server_id, "#channel", "*!*@host.com")
```

#### User Management

```python
# User information
my_nick = irc.current_nick(server_id)
user_info = irc.user_info(server_id, "nickname")
is_online = irc.is_online(server_id, "nickname")

# User operations
irc.change_nick(server_id, "NewNick")
irc.set_away(server_id, "Away message")
irc.whois(server_id, "nickname")
```

#### Command Registration

```python
# Register custom commands
irc.register_command(
    name="mycommand",
    handler=my_handler_function,
    help="Description of the command",
    min_args=0,
    admin_only=False
)

irc.unregister_command("mycommand")

# List registered commands
commands = irc.list_commands()
```

### Event Handlers

Python scripts can handle various IRC events by defining specific functions:

#### Connection Events

```python
def on_connect(server_id):
    """Called when connected to a server"""
    print(f"Connected to server {server_id}")
    
def on_disconnect(server_id, reason):
    """Called when disconnected from a server"""
    print(f"Disconnected: {reason}")
    
def on_error(server_id, error):
    """Called on connection errors"""
    print(f"Error: {error}")
```

#### Message Events

```python
def on_message(server_id, target, nick, message):
    """Handle PRIVMSG"""
    # target is channel or your nick for private messages
    pass

def on_notice(server_id, target, nick, message):
    """Handle NOTICE"""
    pass

def on_action(server_id, target, nick, action):
    """Handle /me actions"""
    pass

def on_ctcp(server_id, nick, command, params):
    """Handle CTCP requests"""
    if command == "VERSION":
        return "MyBot v1.0"  # Return response
    return None  # No response
```

#### Channel Events

```python
def on_join(server_id, channel, nick, account=None, realname=None):
    """User joined channel"""
    if nick != irc.current_nick(server_id):
        irc.send_message(server_id, channel, f"Welcome, {nick}!")

def on_part(server_id, channel, nick, reason=None):
    """User left channel"""
    pass

def on_quit(server_id, nick, reason):
    """User quit IRC"""
    pass

def on_kick(server_id, channel, kicked, kicker, reason=None):
    """User was kicked"""
    pass

def on_nick_change(server_id, old_nick, new_nick):
    """User changed nickname"""
    pass

def on_topic_change(server_id, channel, new_topic, setter):
    """Channel topic changed"""
    pass

def on_mode_change(server_id, target, modes, args, setter):
    """Mode changed"""
    # modes: "+o-v"
    # args: ["nick1", "nick2"]
    pass
```

#### Raw IRC Events

```python
def on_raw(server_id, prefix, command, params):
    """Handle raw IRC messages"""
    # Return True to prevent default handling
    if command == "001":  # RPL_WELCOME
        print("Got welcome message")
    return False
    
def on_numeric(server_id, numeric, params):
    """Handle numeric replies"""
    if numeric == 353:  # RPL_NAMREPLY
        channel = params[2]
        users = params[3].split()
        print(f"Users in {channel}: {users}")
```

### Utility Modules

#### Storage Module

```python
import storage

# Persistent key-value storage
storage.set("last_seen", {"nick": "user", "time": time.time()})
data = storage.get("last_seen", default={})
storage.delete("old_key")
storage.clear()  # Clear all data for this script

# List operations
storage.list_append("users", "new_user")
users = storage.list_get("users", [])
storage.list_remove("users", "old_user")

# JSON storage
storage.save_json("config.json", config_dict)
config = storage.load_json("config.json", {})
```

#### Timer Module

```python
import timer

# One-shot timer
timer_id = timer.once(5.0, callback_function, arg1, arg2)

# Repeating timer
timer_id = timer.interval(60.0, check_updates)

# Cancel timer
timer.cancel(timer_id)

# With lambda
timer.once(10.0, lambda: irc.send_message(server, channel, "Delayed message"))
```

#### HTTP Module

```python
import http

# GET request
def handle_response(response):
    if response.status == 200:
        data = response.json()  # or response.text
        process_data(data)
    else:
        print(f"HTTP error: {response.status}")

http.get("https://api.example.com/data", callback=handle_response)

# POST request
http.post(
    "https://api.example.com/submit",
    json={"key": "value"},
    headers={"Authorization": "Bearer token"},
    callback=handle_post_response
)

# Synchronous requests (blocks event loop - use sparingly)
response = http.get_sync("https://api.example.com/data", timeout=5.0)
```

#### UI Module

```python
import ui

# Print to buffers
ui.print("Message to current buffer")
ui.print_error("Error message")
ui.print_to_server(server_id, "Server message")
ui.print_to_channel(server_id, "#channel", "Channel message")

# Notifications
ui.notify("Script Alert", "Something happened!")
ui.set_status("Script running...")

# User interaction
def handle_input(response):
    if response:
        process_user_input(response)

ui.prompt("Enter value:", callback=handle_input)

# Buffer management
buffer_id = ui.create_buffer("MyScript", server_id)
ui.switch_to_buffer(buffer_id)
ui.clear_buffer(buffer_id)
```

## Advanced Features

### Async/Await Support

```python
import asyncio
import airc  # Async IRC module

async def async_weather_command(args):
    """Async command handler"""
    city = " ".join(args) or "London"
    
    # Async HTTP request
    response = await airc.http_get(f"https://api.weather.com/{city}")
    data = response.json()
    
    # Send response
    await airc.send_message(
        airc.current_server(),
        airc.current_target(),
        f"Weather in {city}: {data['temp']}°C"
    )

# Register async command
airc.register_async_command("weather", async_weather_command)

# Async event handler
async def on_message_async(server_id, target, nick, message):
    """Async message handler"""
    if "http" in message:
        # Extract and fetch URL asynchronously
        url = extract_url(message)
        title = await fetch_url_title(url)
        await airc.send_message(server_id, target, f"Title: {title}")
```

### Database Integration

```python
import db

# SQLite database access
conn = db.connect("script_data.db")

# Create tables
conn.execute("""
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        nick TEXT UNIQUE,
        last_seen INTEGER,
        message_count INTEGER DEFAULT 0
    )
""")

# Prepared statements
stmt = conn.prepare("INSERT OR REPLACE INTO users (nick, last_seen) VALUES (?, ?)")
stmt.execute(nick, int(time.time()))

# Queries
results = conn.query("SELECT * FROM users ORDER BY message_count DESC LIMIT 10")
for row in results:
    print(f"{row['nick']}: {row['message_count']} messages")

# Transactions
with conn.transaction():
    conn.execute("UPDATE users SET message_count = message_count + 1 WHERE nick = ?", nick)
    conn.execute("INSERT INTO messages (nick, message, timestamp) VALUES (?, ?, ?)", 
                 nick, message, time.time())
```

### Inter-Script Communication

```python
import events

# Emit custom events
events.emit("user_score_changed", {"nick": "user", "score": 100})

# Listen for events from other scripts
@events.on("bot_command")
def handle_bot_command(data):
    command = data["command"]
    args = data["args"]
    process_command(command, args)

# Call functions in other scripts
try:
    result = irc.call_script_function("other_script", "get_data", arg1, arg2)
except Exception as e:
    print(f"Failed to call other script: {e}")
```

### Configuration Management

```python
import config

# Define configuration schema
config.define({
    "enabled": {
        "type": bool,
        "default": True,
        "description": "Enable the script"
    },
    "channels": {
        "type": list,
        "default": ["#general"],
        "description": "Channels to monitor"
    },
    "timeout": {
        "type": int,
        "default": 30,
        "min": 1,
        "max": 300,
        "description": "Timeout in seconds"
    }
})

# Access configuration
enabled = config.get("enabled")
config.set("timeout", 60)

# Watch for changes
@config.on_change("enabled")
def on_enabled_change(old_value, new_value):
    if new_value:
        start_monitoring()
    else:
        stop_monitoring()
```

## Security Model

### Sandboxing

Python scripts run in a restricted environment with the following limitations:

```python
# Restricted modules (not available):
# - os (except safe functions)
# - subprocess
# - socket (use irc module instead)
# - threading (use async/await)
# - multiprocessing
# - ctypes
# - importlib (restricted)

# Available safe modules:
# - re, json, time, datetime, math, random
# - collections, itertools, functools
# - hashlib, base64, urllib.parse
# - typing, enum, dataclasses

# File system access is restricted to:
# - Script's own directory
# - Designated data directory
# - No access to system files
```

### Resource Limits

```python
# CPU time limit: 5 seconds per event handler
# Memory limit: 100MB per script
# File size limit: 10MB per file
# Network requests: 60 per minute
# Timers: Maximum 10 active timers

# Scripts exceeding limits are automatically terminated
```

### Permission System

```python
# Script manifest (script.json)
{
    "name": "Advanced Bot",
    "version": "2.0.0",
    "permissions": [
        "network",      # HTTP requests
        "storage",      # Persistent storage
        "notifications", # System notifications
        "database",     # Database access
        "ipc"          # Inter-script communication
    ],
    "python_version": ">=3.8",
    "dependencies": [
        "requests==2.28.0",
        "beautifulsoup4==4.11.0"
    ]
}
```

## Best Practices

### Performance Optimization

```python
# 1. Use async operations for I/O
async def fetch_data():
    return await airc.http_get(url)

# 2. Cache expensive operations
@functools.lru_cache(maxsize=100)
def expensive_calculation(param):
    return complex_operation(param)

# 3. Batch operations
messages = []
for item in items:
    messages.append(format_message(item))
# Send all at once
irc.send_messages(server_id, target, messages)

# 4. Use generators for large datasets
def process_large_file():
    with open("large.txt") as f:
        for line in f:  # Generator, doesn't load entire file
            yield process_line(line)
```

### Error Handling

```python
import logging

# Configure logging
logger = logging.getLogger(__name__)

def safe_handler(func):
    """Decorator for safe event handling"""
    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        try:
            return func(*args, **kwargs)
        except Exception as e:
            logger.error(f"Error in {func.__name__}: {e}", exc_info=True)
            # Optionally notify user
            ui.print_error(f"Script error: {e}")
    return wrapper

@safe_handler
def on_message(server_id, target, nick, message):
    # Your code here - exceptions will be caught
    process_message(message)
```

### Testing Scripts

```python
# test_mybot.py
import unittest
from unittest.mock import Mock, patch
import mybot

class TestMyBot(unittest.TestCase):
    def setUp(self):
        self.server_id = 1
        self.channel = "#test"
        
    @patch('irc.send_message')
    def test_hello_command(self, mock_send):
        mybot.cmd_hello(["World"])
        mock_send.assert_called_with(
            self.server_id, 
            self.channel, 
            "Hello, World!"
        )
    
    def test_config_parsing(self):
        result = mybot.parse_config("key=value")
        self.assertEqual(result, {"key": "value"})

if __name__ == "__main__":
    unittest.main()
```

## Example Scripts

### RSS Feed Monitor

```python
"""
rss_monitor.py - Monitor RSS feeds and post updates
"""

__name__ = "RSS Monitor"
__version__ = "1.0.0"

import irc
import http
import storage
import timer
from datetime import datetime
import xml.etree.ElementTree as ET

config = {
    "feeds": {
        "https://example.com/rss": ["#news"],
        "https://blog.example.com/feed": ["#tech", "#general"]
    },
    "check_interval": 300,  # 5 minutes
    "max_items": 5
}

def on_load():
    """Initialize RSS monitor"""
    # Start monitoring
    check_feeds()
    
    # Schedule regular checks
    timer.interval(config["check_interval"], check_feeds)
    
    # Register commands
    irc.register_command("rss", cmd_rss, "Manage RSS feeds")

def check_feeds():
    """Check all configured feeds"""
    for feed_url, channels in config["feeds"].items():
        http.get(feed_url, callback=lambda resp: process_feed(resp, channels))

def process_feed(response, channels):
    """Process RSS feed response"""
    if response.status != 200:
        return
    
    try:
        root = ET.fromstring(response.text)
        items = root.findall(".//item")[:config["max_items"]]
        
        # Get last check time
        last_check = storage.get(f"last_check_{response.url}", 0)
        new_items = []
        
        for item in items:
            pub_date = parse_date(item.findtext("pubDate"))
            if pub_date > last_check:
                new_items.append({
                    "title": item.findtext("title"),
                    "link": item.findtext("link"),
                    "date": pub_date
                })
        
        # Post new items
        if new_items:
            server = irc.current_server()
            for channel in channels:
                for item in reversed(new_items):  # Post oldest first
                    message = f"📰 {item['title']} - {item['link']}"
                    irc.send_message(server, channel, message)
            
            # Update last check time
            storage.set(f"last_check_{response.url}", datetime.now().timestamp())
            
    except Exception as e:
        print(f"Error processing feed: {e}")

def cmd_rss(args):
    """Handle RSS commands"""
    if not args:
        return "Usage: !rss <list|add|remove> [url] [channels...]"
    
    action = args[0].lower()
    
    if action == "list":
        if not config["feeds"]:
            return "No feeds configured"
        
        lines = ["Configured RSS feeds:"]
        for url, channels in config["feeds"].items():
            lines.append(f"  {url} → {', '.join(channels)}")
        return "\n".join(lines)
    
    elif action == "add" and len(args) >= 3:
        url = args[1]
        channels = args[2:]
        config["feeds"][url] = channels
        storage.save_json("rss_config.json", config)
        return f"Added feed: {url}"
    
    elif action == "remove" and len(args) >= 2:
        url = args[1]
        if url in config["feeds"]:
            del config["feeds"][url]
            storage.save_json("rss_config.json", config)
            return f"Removed feed: {url}"
        return "Feed not found"
```

### Game Bot

```python
"""
trivia_bot.py - Trivia game for IRC channels
"""

__name__ = "Trivia Bot"
__version__ = "2.0.0"

import irc
import timer
import storage
import random
from datetime import datetime

class TriviaGame:
    def __init__(self):
        self.active = False
        self.current_question = None
        self.channel = None
        self.server = None
        self.scores = {}
        self.question_timer = None
        self.questions = self.load_questions()
    
    def load_questions(self):
        """Load trivia questions from storage"""
        default_questions = [
            {
                "question": "What year was Python created?",
                "answer": "1991",
                "category": "Technology",
                "difficulty": "medium"
            },
            {
                "question": "Who created the Linux kernel?",
                "answer": "Linus Torvalds",
                "category": "Technology",
                "difficulty": "easy"
            }
            # Add more questions
        ]
        return storage.load_json("trivia_questions.json", default_questions)
    
    def start(self, server, channel):
        """Start a new game"""
        if self.active:
            return "A game is already in progress!"
        
        self.active = True
        self.server = server
        self.channel = channel
        self.scores = {}
        
        irc.send_message(server, channel, 
            "🎮 Trivia game starting! Type your answers in the channel.")
        
        timer.once(3.0, self.ask_question)
        return None
    
    def ask_question(self):
        """Ask a new question"""
        if not self.active:
            return
        
        self.current_question = random.choice(self.questions)
        q = self.current_question
        
        message = (f"❓ [{q['category']} - {q['difficulty'].title()}] "
                  f"{q['question']}")
        irc.send_message(self.server, self.channel, message)
        
        # Set timeout for this question
        self.question_timer = timer.once(30.0, self.timeout_question)
    
    def check_answer(self, nick, answer):
        """Check if answer is correct"""
        if not self.current_question:
            return False
        
        correct = self.current_question["answer"].lower()
        if answer.lower() == correct:
            # Cancel timeout
            if self.question_timer:
                timer.cancel(self.question_timer)
            
            # Update score
            self.scores[nick] = self.scores.get(nick, 0) + 1
            
            irc.send_message(self.server, self.channel,
                f"✅ Correct, {nick}! Your score: {self.scores[nick]}")
            
            # Next question after delay
            timer.once(3.0, self.ask_question)
            self.current_question = None
            return True
        
        return False
    
    def timeout_question(self):
        """Handle question timeout"""
        if self.current_question:
            answer = self.current_question["answer"]
            irc.send_message(self.server, self.channel,
                f"⏰ Time's up! The answer was: {answer}")
            
            self.current_question = None
            timer.once(3.0, self.ask_question)
    
    def stop(self):
        """Stop the game"""
        if not self.active:
            return "No game in progress"
        
        self.active = False
        if self.question_timer:
            timer.cancel(self.question_timer)
        
        # Show final scores
        if self.scores:
            scores = sorted(self.scores.items(), 
                          key=lambda x: x[1], reverse=True)
            
            lines = ["🏆 Final scores:"]
            for i, (nick, score) in enumerate(scores[:5], 1):
                lines.append(f"{i}. {nick}: {score} points")
            
            message = "\n".join(lines)
        else:
            message = "Game ended. No scores recorded."
        
        irc.send_message(self.server, self.channel, message)
        return None

# Global game instance
game = TriviaGame()

def on_load():
    """Initialize trivia bot"""
    irc.register_command("trivia", cmd_trivia, "Trivia game commands")
    print(f"[{__name__}] Loaded with {len(game.questions)} questions")

def on_message(server_id, target, nick, message):
    """Check answers during active game"""
    if game.active and target == game.channel:
        game.check_answer(nick, message)

def cmd_trivia(args):
    """Handle trivia commands"""
    if not args:
        return "Usage: !trivia <start|stop|scores|add>"
    
    action = args[0].lower()
    server = irc.current_server()
    channel = irc.current_target()
    
    if action == "start":
        return game.start(server, channel)
    elif action == "stop":
        return game.stop()
    elif action == "scores":
        if not game.scores:
            return "No scores yet"
        scores = sorted(game.scores.items(), 
                       key=lambda x: x[1], reverse=True)
        lines = ["Current scores:"]
        for nick, score in scores[:5]:
            lines.append(f"{nick}: {score}")
        return "\n".join(lines)
    elif action == "add" and len(args) >= 4:
        # !trivia add category difficulty question|answer
        category = args[1]
        difficulty = args[2]
        qa = " ".join(args[3:]).split("|")
        if len(qa) == 2:
            game.questions.append({
                "question": qa[0].strip(),
                "answer": qa[1].strip(),
                "category": category,
                "difficulty": difficulty
            })
            storage.save_json("trivia_questions.json", game.questions)
            return "Question added!"
        return "Format: !trivia add category difficulty question|answer"
```

## Debugging and Development

### Debug Mode

```python
# Enable debug mode for your script
import irc
irc.set_debug(True)

# Debug output
irc.debug("Variable state:", variable)
irc.debug_event("on_message", locals())

# Performance profiling
from irc.profile import profile

@profile
def expensive_function():
    # Your code here
    pass

# Get profiling results
results = irc.get_profile_data()
```

### Development Tools

```python
# REPL for testing
# /script console script_name

# Reload script without unloading
# /script reload script_name

# View script logs
# /script logs script_name

# Script inspector
import inspector

# List all available functions
funcs = inspector.list_functions(irc)

# Get function signature
sig = inspector.signature(irc.send_message)

# View script state
state = inspector.get_script_state()
```

## Distribution and Sharing

### Package Structure

```
my_script/
├── __init__.py       # Main script file
├── script.json       # Manifest file
├── requirements.txt  # Python dependencies
├── README.md         # Documentation
├── LICENSE          # License file
├── config.default.json  # Default configuration
└── modules/         # Additional modules
    ├── utils.py
    └── handlers.py
```

### Script Repository

Scripts can be shared through the RustIRC script repository:

```python
# Package script
# /script package my_script

# Publish to repository
# /script publish my_script

# Install from repository
# /script install script_name

# Update installed scripts
# /script update
```

## Migration from Other Clients

### From HexChat

```python
# HexChat compatibility layer
import hexchat_compat as hexchat

# Familiar API
hexchat.hook_print("Channel Message", on_message)
hexchat.hook_command("mycommand", on_command)
hexchat.command("MSG #channel Hello!")

# Get context
context = hexchat.get_context()
hexchat.set_context(context)
```

### From WeeChat

```python
# WeeChat compatibility layer
import weechat_compat as weechat

# Similar API structure
weechat.register("myscript", "author", "1.0", "GPL3", "Description", "", "")
weechat.hook_print("", "irc_privmsg", "", 1, "on_message", "")
weechat.command("", "/msg #channel Hello!")
```

## Troubleshooting

### Common Issues

1. **Script not loading**
   ```python
   # Check Python version
   import sys
   print(f"Python {sys.version}")
   
   # Verify metadata
   assert __name__ and __version__
   ```

2. **Import errors**
   ```python
   # Use try/except for optional imports
   try:
       import advanced_module
       HAS_ADVANCED = True
   except ImportError:
       HAS_ADVANCED = False
   ```

3. **Performance issues**
   ```python
   # Use async operations
   # Limit timer frequency
   # Cache expensive operations
   # Profile your code
   ```

### Getting Help

- Documentation: https://docs.rustirc.org/python
- Examples: https://github.com/rustirc/python-scripts
- Community: #rustirc on Libera.Chat
- Python-specific channel: #rustirc-python