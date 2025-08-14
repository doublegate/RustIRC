# ADR-003: Plugin Architecture

## Status
Accepted

## Context
Extensibility is crucial for IRC clients. Users need to customize behavior, add features, and integrate with external services. The plugin architecture must balance power, safety, and ease of use.

## Decision
We will implement a **hybrid plugin system**:
1. **Lua scripting** via mlua for lightweight extensions
2. **Native Rust plugins** via dynamic libraries for performance-critical features
3. **Future: Python support** via PyO3 for data science/ML integrations

## Architecture

### Event-Driven API
Plugins register handlers for events:
- Message events (PRIVMSG, NOTICE, JOIN, etc.)
- Client events (connect, disconnect, error)
- UI events (tab switch, input, click)

### Sandboxing Strategy
- Lua scripts run in sandboxed environment with:
  - No file I/O by default
  - Memory limits
  - CPU time limits
  - Restricted module access
- Native plugins require user approval
- Capability-based permission system

### Plugin Lifecycle
1. Discovery (scan directories)
2. Loading (parse metadata)
3. Initialization (setup environment)
4. Registration (event handlers)
5. Execution (handle events)
6. Shutdown (cleanup)

## Consequences

### Positive
- Lua provides safe, easy scripting for most use cases
- Native plugins enable high-performance extensions
- Event-driven model matches IRC's nature
- Sandboxing protects against malicious scripts

### Negative
- Complexity of maintaining multiple plugin types
- Native plugins can crash the client
- Sandbox limitations may frustrate power users

## API Design

### Core Functions
```lua
irc.send_message(target, text)
irc.send_raw(command)
irc.join_channel(channel)
irc.part_channel(channel)
irc.get_nick()
irc.get_channels()
```

### Event Handlers
```lua
irc.on("PRIVMSG", function(msg)
  -- Handle private message
end)

irc.on("JOIN", function(msg)
  -- Handle join event
end)
```

### Storage API
```lua
irc.store_set(key, value)
irc.store_get(key)
irc.store_delete(key)
```

## Security Model

### Permissions
- Core: Basic IRC operations
- Network: HTTP requests
- FileSystem: Read/write files
- Process: Spawn processes
- UI: Modify interface

### Default Policies
- Lua scripts: Core only
- Signed plugins: Core + Network
- Native plugins: User approval required

## Validation
Scripting prototype demonstrated:
- Event handling with <1ms overhead
- Successful sandboxing preventing file access
- Memory limits enforced correctly