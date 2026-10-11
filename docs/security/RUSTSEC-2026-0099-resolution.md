# Security Advisory Resolution: RUSTSEC-2026-0099

## Summary

Verified that RustIRC is **NOT VULNERABLE** to RUSTSEC-2026-0099. The project uses `rustls-webpki` version 0.103.15, which fixes the incorrect acceptance of permitted subtree name constraints for certificates asserting wildcard names.

- **GitHub Issue**: [#103](https://github.com/doublegate/RustIRC/issues/103)
- **Advisory**: RUSTSEC-2026-0099
- **Package**: `rustls-webpki`
- **Affected Versions**: `< 0.103.12`
- **Patched Versions**: `>= 0.103.12, < 0.104.0-alpha.1` OR `>= 0.104.0-alpha.6`
- **Current Version in RustIRC**: `0.103.15` ✅
- **Severity**: Medium (Name Constraints Bypass)
- **Date Verified**: 2026-10-11

## Vulnerability Details

In vulnerable versions of `rustls-webpki`, permitted subtree name constraints for DNS names were accepted for certificates asserting a wildcard name (`*.example.com`).

Given a name constraint of `accept.example.com`, a wildcard certificate for `*.example.com` could feasibly match `reject.example.com`, violating the constraint boundary (similar to CVE-2025-61727).

## Current Status in RustIRC

### Dependency Chain

`rustls-webpki` is used via `rustls` 0.23.45 across all network-facing crates.

### Verification Evidence

1. **Cargo.lock Check**:
   `rustls-webpki` is locked at `0.103.15`, exceeding the minimum patched version `0.103.12`.
2. **Workspace Requirement**:
   `rustls = "0.23.45"` is enforced in `Cargo.toml`.
3. **Automated Test**:
   `test_rustls_webpki_root_store_and_server_names` is a configuration smoke test only; it does not validate chains, name constraints, or CRLs. `test_security_advisory_dependency_floors` rejects vulnerable resolved webpki versions. Advisory-specific certificate/CRL fixtures are not supplied by this PR.
4. **Security Audit**:
   `cargo audit` passes cleanly with 0 vulnerabilities detected.

## Conclusion

RustIRC is secure against RUSTSEC-2026-0099. Issue #103 is fully resolved.
