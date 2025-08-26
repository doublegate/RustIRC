# ADR-001: GUI Framework Selection

## Status
Accepted

## Context
RustIRC requires a cross-platform GUI framework that can deliver a modern, responsive interface while maintaining good performance with large amounts of IRC data.

## Decision
We will use **Iced** as our primary GUI framework.

## Consequences

### Positive
- Pure Rust implementation (no C/C++ dependencies)
- Elm-inspired architecture fits well with IRC's event-driven nature
- Good performance with virtual scrolling for large buffers
- Active development and growing community
- Built-in theming support
- Native look on all platforms without platform-specific code

### Negative
- Relatively young compared to GTK
- Smaller ecosystem of pre-built widgets
- Documentation still evolving

## Alternatives Considered

### GTK-rs
- **Pros**: Mature, extensive widget library, native Linux look
- **Cons**: C dependencies, complex cross-platform builds, heavyweight, platform inconsistencies

## Branch Strategy

### Current Implementation
- **Main Branch**: Continues with Iced 0.13.1 implementation
- **Dioxus Branch**: Experimental Dioxus v0.6 implementation 
- **Evaluation Criteria**: Development experience, performance, maintainability, feature set

### Decision Timeline
This branch serves as a proof-of-concept to evaluate whether Dioxus provides sufficient advantages to justify migration from the stable Iced implementation.

### egui
- **Pros**: Immediate mode, simple API, good for tools
- **Cons**: Less suitable for complex application UIs, limited theming

### Tauri
- **Pros**: Web technologies, rich ecosystem
- **Cons**: Overhead of web renderer, not pure Rust

### Dioxus v0.6 (This Branch - Under Evaluation)
- **Pros**: React-like development, hot reload, WebView rich content, hooks-based state, mobile future
- **Cons**: WebView dependencies, different paradigm, bundle overhead
- **Status**: Experimental implementation for comparison with Iced

## Validation
Prototype testing showed Iced can handle 10,000+ messages with smooth scrolling and maintains 60 FPS during normal operation.