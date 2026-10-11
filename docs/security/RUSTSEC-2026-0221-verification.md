# Security Verification: RUSTSEC-2026-0221

## Summary

Verified that RustIRC is **NOT VULNERABLE** to RUSTSEC-2026-0221. The project uses `event-listener` version 5.4.2, which patches the unsound `Send` and `Sync` implementations for `StackSlot<'_, T>`.

- **GitHub Issue**: [#137](https://github.com/doublegate/RustIRC/issues/137)
- **Advisory**: RUSTSEC-2026-0221
- **Package**: `event-listener`
- **Affected Versions**: `5.1.0` - `5.4.1`
- **Patched Versions**: `>= 5.4.2`
- **Current Version in RustIRC**: `5.4.2` ✅
- **Severity**: Unsound
- **Date Verified**: 2026-10-11

## Vulnerability Details

In versions of `event-listener` between 5.1.0 and 5.4.1, `StackSlot<'_, T>` (the stack-allocated listener type created by the `listener!` macro) unconditionally implemented `Send` and `Sync`.

This allowed a `!Send` tag type set via `Event::with_tag` to cross thread boundaries and be accessed on another thread via `StackSlot::wait`, resulting in potential data races in safe code.

## Current Status in RustIRC

### Dependency Chain

`event-listener` is present in the dependency tree as a concurrency primitive through `async-broadcast`, `async-lock`, `async-process`, and `zbus`:

```
event-listener v5.4.2
├── async-broadcast v0.7.2
│   └── zbus v5.19.0
│       ├── mundy v0.2.3 -> iced_winit -> iced -> rustirc-gui -> rustirc
│       └── notify-rust v4.18.2 -> rustirc-gui -> rustirc
├── async-lock v3.4.2
└── event-listener-strategy v0.5.4
```

### Verification Evidence

1. **Cargo.lock Check**:
   `event-listener` is locked at version `5.4.2`, which includes upstream fix in PR #163.
2. **Workspace Requirement**:
   `Cargo.toml` declares `event-listener = "5.4.2"` under `[workspace.dependencies]` to prevent downgrades.
3. **Automated Test**:
   `test_rustsec_2026_0221_event_listener_stack_slot_safety` checks notification using `listener!` as a synchronization smoke test. Module-level `static_assertions` separately verify that `StackSlot<Rc<()>>` is neither `Send` nor `Sync`, while `StackSlot<u64>` is both. The negative bounds target the affected type directly rather than relying on an unrelated listener lifetime error.
4. **Security Audit**:
   `cargo audit` passes with 0 vulnerabilities detected.

## Conclusion

RustIRC is fully protected against RUSTSEC-2026-0221. No vulnerability exists.
