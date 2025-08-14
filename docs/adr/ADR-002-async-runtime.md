# ADR-002: Async Runtime Selection

## Status
Accepted

## Context
IRC clients require efficient handling of multiple concurrent network connections, timers, and I/O operations. The async runtime is critical for performance and resource usage.

## Decision
We will use **Tokio** as our async runtime.

## Consequences

### Positive
- Most mature and widely adopted async runtime in Rust ecosystem
- Excellent performance characteristics
- Comprehensive feature set (timers, I/O, channels, sync primitives)
- Strong ecosystem support (most async crates support Tokio)
- Built-in tracing integration
- Multi-threaded by default with work-stealing scheduler

### Negative
- Larger binary size compared to lighter runtimes
- Learning curve for developers new to Tokio
- Some overhead for simple applications (though negligible for IRC client)

## Alternatives Considered

### async-std
- **Pros**: Familiar std-like API, smaller binary
- **Cons**: Smaller ecosystem, less active development

### smol
- **Pros**: Minimal, lightweight
- **Cons**: Less features, would require more custom implementation

### Embassy
- **Pros**: Great for embedded
- **Cons**: Not suitable for desktop applications

## Validation
Network prototype testing demonstrated:
- Handling 100+ concurrent connections without issues
- Sub-millisecond message parsing
- Efficient memory usage under load