# RUSTSEC-2026-0097 Security Verification Report

## Advisory Details

- **Advisory ID**: RUSTSEC-2026-0097
- **Title**: Rand is unsound with a custom logger using `rand::rng()`
- **Severity**: Unsound (potential Undefined Behavior)
- **Date Reported**: April 9, 2026
- **Date Issued**: April 11, 2026

## Vulnerability Summary

The `rand` crate versions >= 0.7.0, < 0.8.6 and >= 0.9.0, < 0.9.3 and 0.10.0 contain an unsoundness vulnerability that can cause Undefined Behavior when ALL of the following conditions are met:

1. The `log` and `thread_rng` features are enabled
2. A custom logger is defined for the `log` crate
3. The custom logger calls `rand::rng()` or `rand::thread_rng()`
4. The `ThreadRng` attempts to reseed (every 64 kB of generated data)
5. Trace or warn-level logging is enabled

## Affected Versions

- >= 0.7.0, < 0.8.6
- >= 0.9.0, < 0.9.3  
- 0.10.0 (exact version)

## Patched Versions

- >= 0.8.6, < 0.9.0
- >= 0.9.3, < 0.10.0
- >= 0.10.1

## RustIRC Project Status

### Current Configuration

**RESULT: ✅ NOT VULNERABLE**

- **Current rand version**: 0.10.3 (patched)
- **Vulnerability fixed in**: 0.10.1
- **log feature enabled**: NO
- **thread_rng feature enabled**: YES
- **Logging framework**: `tracing` (not `log`)

### Verification Results

#### 1. Version Check

```bash
$ cargo tree -i rand@0.10.3
rand v0.10.3
└── rustirc-core v0.4.2 (/workspace/crates/rustirc-core)
```

The project uses rand 0.10.3, which is >= 0.10.1 (patched version).

#### 2. Feature Analysis

```bash
$ cargo tree -e features -i rand@0.10.3
```

Enabled features:
- ✅ `std` - Standard library support
- ✅ `thread_rng` - Thread-local RNG (required for vulnerability)
- ✅ `default` - Default features
- ✅ `alloc` - Allocation support
- ✅ `getrandom` - System RNG
- ✅ `std_rng` - Standard RNG
- ✅ `sys_rng` - System RNG
- ❌ `log` - **NOT ENABLED** (required for vulnerability)

**Critical Finding**: The `log` feature is NOT enabled, which means the project would not be vulnerable even if using rand 0.10.0.

#### 3. Logging Framework

The project uses `tracing` for structured logging, not the `log` crate. Confirmed in dependencies:

```toml
# workspace Cargo.toml
tracing = "0.1.44"
tracing-subscriber = { version = "0.3.23", features = ["env-filter"] }
```

No `log` crate dependency exists in the project.

#### 4. Custom Logger Check

Searched for custom logger implementations:
- No implementation of `log::Log` trait found
- The `LoggerPlugin` is for IRC message logging, not a `log` crate logger
- Uses `tracing` macros throughout the codebase

#### 5. rand Usage

The project uses `rand::rng()` in one location:

```rust
// crates/rustirc-core/src/auth.rs:183
fn generate_scram_nonce() -> String {
    use rand::RngExt;
    let mut rng = rand::rng();
    let random_bytes: [u8; 18] = rng.random();
    BASE64.encode(random_bytes)
}
```

This usage is safe:
- ✅ Not called from a logger
- ✅ Standard SCRAM authentication nonce generation
- ✅ No interaction with logging

#### 6. Test Results

All workspace tests pass successfully:

```bash
$ cargo test --lib --workspace
test result: ok. 233 passed; 0 failed; 0 ignored; 0 measured
```

## Conclusion

The RustIRC project is **NOT VULNERABLE** to RUSTSEC-2026-0097 for multiple reasons:

1. **Primary**: Uses patched version rand 0.10.3 (>= 0.10.1)
2. **Secondary**: Does not enable the `log` feature for rand
3. **Tertiary**: Uses `tracing` instead of `log` crate
4. **Quaternary**: No custom `log` logger implementations exist

## Recommendations

### Current Status
✅ **No action required** - The project is already secure.

### Best Practices Applied
1. ✅ Using latest patched version (0.10.3)
2. ✅ Minimal feature set (no unnecessary features enabled)
3. ✅ Modern logging framework (`tracing` over `log`)
4. ✅ Regular dependency updates (Dependabot enabled)

### Future Maintenance
- Continue monitoring RustSec advisories via `cargo audit`
- Keep rand dependency updated to latest stable versions
- Maintain current minimal feature configuration
- Continue using `tracing` for logging (more features, better performance)

## References

- [RustSec Advisory RUSTSEC-2026-0097](https://rustsec.org/advisories/RUSTSEC-2026-0097)
- [GitHub Advisory GHSA-cq8v-f236-94qc](https://github.com/advisories/GHSA-cq8v-f236-94qc)
- [rand PR #1763](https://github.com/rust-random/rand/pull/1763)
- [OSV Vulnerability Database](https://osv.dev/vulnerability/RUSTSEC-2026-0097)

---

**Verified by**: Cursor Cloud Agent  
**Date**: 2026-10-10  
**Test Suite**: All 233 unit tests passing  
**Build Status**: ✅ Success
