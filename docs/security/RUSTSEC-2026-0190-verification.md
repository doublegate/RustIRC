# Security Verification: RUSTSEC-2026-0190

## Summary

Verified that RustIRC is **NOT VULNERABLE** to RUSTSEC-2026-0190. The project uses `anyhow` version 1.0.104, which resolves the mutable reference aliasing unsoundness in `Error::downcast_mut()`.

- **GitHub Issue**: [#132](https://github.com/doublegate/RustIRC/issues/132)
- **Advisory**: RUSTSEC-2026-0190
- **Package**: `anyhow`
- **Affected Versions**: `<= 1.0.102`
- **Patched Versions**: `>= 1.0.103`
- **Current Version in RustIRC**: `1.0.104` ✅
- **Severity**: Unsound
- **Date Verified**: 2026-10-11

## Vulnerability Details

Affected versions of `anyhow` violated borrow rules (resulting in undefined behavior under Stacked Borrows) when context was added to an error via `Error::context` and later modified via `Error::downcast_mut` on the returned `Error`.

The flaw was corrected in upstream commit `6e8c000` by revising how the mutable reference is constructed, avoiding inclusion of a shared reference in the resulting borrow chain.

## Current Status in RustIRC

### Dependency Chain

`anyhow` is a primary workspace error handling crate used across `rustirc-core`, `rustirc-gui`, `rustirc-plugins`, `rustirc-protocol`, `rustirc-scripting`, and `rustirc-tui`.

### Verification Evidence

1. **Cargo.lock Check**:
   `anyhow` is locked at version `1.0.104`, above the `1.0.103` patch threshold.
2. **Workspace Requirement**:
   `Cargo.toml` explicitly sets `anyhow = "1.0.104"` in `[workspace.dependencies]`.
3. **Automated Test**:
   `tests/security_advisories_test.rs::test_rustsec_2026_0190_anyhow_downcast_mut_soundness` checks context mutation and formatted output as a behavioral smoke test. Ordinary assertions do not prove soundness or absence of memory corruption. Protection relies on the patched dependency and upstream fix; `test_security_advisory_dependency_floors` rejects vulnerable lockfile versions.
4. **Security Audit**:
   `cargo audit` passes cleanly with zero vulnerabilities.

## Conclusion

RustIRC is fully protected against RUSTSEC-2026-0190.

## Optional interpreter checks

Use a nightly toolchain with the Miri component installed for additional UB checking:

```bash
cargo +nightly miri test --test security_advisories_test test_rustsec_2026_0190_anyhow_downcast_mut_soundness
cargo +nightly miri test --test security_advisories_test test_rustsec_2026_0204_crossbeam_epoch_pointer_safety
```

These commands supplement standard Cargo tests; they have not been run as part of this verification. Miri support for the full GUI dependency graph may require a reduced upstream reproducer.
