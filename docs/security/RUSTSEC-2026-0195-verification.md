# Security Verification: RUSTSEC-2026-0195

## Summary

Verified that RustIRC is **NOT VULNERABLE** to RUSTSEC-2026-0195. The project uses `quick-xml` version 0.41.0, which resolves the unbounded namespace-declaration allocation vulnerability in `NsReader`.

- **GitHub Issue**: [#130](https://github.com/doublegate/RustIRC/issues/130)
- **Advisory**: RUSTSEC-2026-0195
- **Package**: `quick-xml`
- **Affected Versions**: `< 0.41.0`
- **Patched Versions**: `>= 0.41.0`
- **Current Version in RustIRC**: `0.41.0` ✅
- **Severity**: High (Memory exhaustion Denial of Service)
- **Date Verified**: 2026-10-11

## Vulnerability Details

In `quick-xml < 0.41.0`, `NsReader` allocated unbounded heap memory when processing namespace declarations. For every `Start` or `Empty` event, `NamespaceResolver::push` would iterate all `xmlns` and `xmlns:*` attributes and allocate memory without an upper bound. This enabled remote attackers supplying untrusted XML to cause Out-Of-Memory (OOM) conditions.

## Resolution in quick-xml 0.41.0

The patched version introduces bounded namespace declarations:
- Rejects start tags exceeding `DEFAULT_MAX_DECLARATIONS_PER_ELEMENT` (256).
- Provides configurable limits via `NamespaceResolver::set_max_declarations_per_element`.
- Returns `NamespaceError::TooManyDeclarations` when limits are exceeded.

## Current Status in RustIRC

### Dependency Chain

`quick-xml` is pulled in transitively via `wayland-scanner` in the Linux GUI stack.

### Verification Evidence

1. **Cargo.lock Check**:
   `quick-xml` is pinned to `0.41.0`.
2. **Workspace Requirement**:
   `quick-xml = "0.41.0"` is declared in `[workspace.dependencies]`.
3. **Automated Test**:
   `tests/security_advisories_test.rs::test_rustsec_2026_0195_quick_xml_ns_reader_bounded` validates `NsReader` namespace parsing on XML inputs.
4. **Security Audit**:
   `cargo audit` completes cleanly with 0 vulnerabilities detected.

## Conclusion

RustIRC is secure against RUSTSEC-2026-0195. Issue #130 is fully resolved.
