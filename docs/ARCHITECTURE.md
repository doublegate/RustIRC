# RustIRC Architecture

## Overview

RustIRC follows a modular, event-driven architecture designed for extensibility, performance, and maintainability. The system is built as a collection of loosely coupled crates that communicate through well-defined interfaces.

## System Architecture

```text
┌─────────────────────────────────────────────────────────┐
│                         User Interface                  │
│  ┌─────────────────┐              ┌──────────────────┐  │
│  │   GUI (Iced)    │              │   TUI (Ratatui)  │  │
│  └────────┬────────┘              └────────┬─────────┘  │
└───────────┼────────────────────────────────┼────────────┘
            │                                │
┌───────────▼────────────────────────────────▼───────────┐
│                         Core Client                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │Event Bus     │  │State Manager │  │Config Manager│  │
│  └──────┬───────┘  └──────┬───────┘  └──────────────┘  │
│         │                  │                           │
│  ┌──────▼───────┐  ┌──────▼───────┐  ┌──────────────┐  │
│  │Connection Mgr│  │Buffer Manager│  │Command Queue │  │
│  └──────┬───────┘  └──────────────┘  └──────────────┘  │
└─────────┼──────────────────────────────────────────────┘
          │
┌─────────▼────────────────────────────────────────────────┐
│                      Protocol Layer                      │
│  ┌──────────────┐  ┌───────────────┐  ┌───────────────┐  │
│  │Message Parser│  │Message Builder│  │Capability Mgr │  │
│  └──────────────┘  └───────────────┘  └───────────────┘  │
└──────────────────────────────────────────────────────────┘
          │
┌─────────▼──────────────────────────────────────────────┐
│                      Network Layer                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │TCP/TLS Socket│  │DCC Handler   │  │SASL Auth     │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└────────────────────────────────────────────────────────┘
          │
┌─────────▼──────────────────────────────────────────────┐
│                    Extension System                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │Lua Scripts   │  │Python Scripts│  │Native Plugins│  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Event Bus

- Central message routing system
- Priority-based event handlers
- Async event processing
- Plugin event injection points

### 2. State Manager

- Immutable state transitions
- Event sourcing for history
- Subscription-based updates
- State persistence/restoration

### 3. Connection Manager

- Multi-server connections
- Connection pooling
- Automatic reconnection
- Network state tracking

### 4. Buffer Manager

- Message history management
- Virtual scrolling for performance
- Search and filtering
- Log persistence

### 5. Protocol Handler

- IRC message parsing/building
- IRCv3 capability negotiation
- SASL authentication
- DCC protocol support

### 6. Extension System

- Lua scripting engine
- Python integration (future)
- Native plugin loading
- Sandboxed execution

## Data Flow

### Incoming Messages

1. Network socket receives data
2. Protocol parser validates and parses message
3. Message converted to internal event
4. Event published to event bus
5. Handlers process event in priority order
6. State manager updates state
7. UI components receive state updates
8. Plugins/scripts receive filtered events

### Outgoing Messages

1. User action or script triggers command
2. Command validated and queued
3. Message builder creates IRC message
4. Rate limiting applied if needed
5. Message sent through network socket
6. Confirmation event generated
7. State updated with sent message

## Crate Structure

### rustirc-core

Core client functionality:

- Client manager
- Event system
- State management
- Configuration

### rustirc-protocol

IRC protocol implementation:

- Message parsing/building
- Command definitions
- Numeric replies
- IRCv3 capabilities

### rustirc-gui

GUI implementation using Iced:

- Main application window
- Custom widgets
- Theme management
- Platform integration

### rustirc-tui

TUI implementation using Ratatui:

- Terminal rendering
- Keyboard input handling
- Buffer visualization
- Status displays

### rustirc-scripting

Scripting engine:

- Lua runtime management
- Script API
- Sandboxing
- Resource limits

### rustirc-plugins

Plugin system:

- Plugin loading
- API definitions
- IPC communication
- Resource management

## Design Patterns

### Event-Driven Architecture

All components communicate through events, enabling:

- Loose coupling between modules
- Easy testing and mocking
- Plugin integration points
- Async processing

### Command Pattern

User actions and script commands use command pattern:

- Undoable operations
- Command queuing
- Rate limiting
- Validation

### Observer Pattern

State changes notify observers:

- UI updates
- Plugin notifications
- Logging
- Persistence

### Strategy Pattern

Pluggable algorithms for:

- Authentication methods
- Encoding strategies
- Connection methods
- UI rendering

## Security Architecture

### Network Security

- TLS by default
- Certificate validation
- Certificate pinning support
- SASL authentication

### Script Security

- Sandboxed execution
- Resource limits
- Permission system
- No direct file/network access

### Plugin Security

- Process isolation (future)
- Capability-based permissions
- Signed plugins (future)
- Resource quotas

## Performance Considerations

### Memory Management

- Lazy loading of history
- Virtual scrolling
- Buffer size limits
- Message compression

### CPU Optimization

- Async I/O throughout
- Work stealing scheduler
- Parser optimization
- Batch processing

### Network Efficiency

- Connection pooling
- Message batching
- Compression support
- Smart reconnection

## Extension Points

### Event Hooks

Plugins can hook into:

- Pre/post message processing
- State changes
- UI events
- Network events

### Custom Commands

Extensions can register:

- Slash commands
- Aliases
- Keyboard shortcuts
- Context menu items

### UI Integration

Plugins can:

- Add custom widgets
- Create dialogs
- Modify themes
- Add status indicators

## Configuration

### Layered Configuration

1. Default values
2. System configuration
3. User configuration
4. Runtime overrides

### Configuration Sources

- TOML files
- Environment variables
- Command-line arguments
- UI preferences

## Testing Strategy

### Unit Testing

- Individual component testing
- Mock implementations
- Property-based testing
- Fuzzing

### Integration Testing

- Multi-component interaction
- Protocol compliance
- Network simulation
- Plugin integration

### Performance Testing

- Load testing
- Memory profiling
- Latency measurement
- Throughput benchmarks

## Future Considerations

### Planned Features

- Web UI support
- Mobile companion app
- Cloud synchronization
- Voice/video integration

### Scalability

- Clustering support
- Distributed state
- Load balancing
- Horizontal scaling

### Platform Support

- WASM compilation
- Mobile platforms
- Embedded systems
- Cloud deployment
