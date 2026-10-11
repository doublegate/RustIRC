# Security Advisory Resolution: RUSTSEC-2026-0049

## Summary

Verified that RustIRC is **NOT VULNERABLE** to RUSTSEC-2026-0049. The project uses `rustls-webpki` version 0.103.15, which fixes the CRL Distribution Point matching logic bug.

- **GitHub Issue**: [#90](https://github.com/doublegate/RustIRC/issues/90)
- **Advisory**: RUSTSEC-2026-0049
- **Package**: `rustls-webpki`
- **Affected Versions**: `< 0.103.10`
- **Patched Versions**: `>= 0.103.10`
- **Current Version in RustIRC**: `0.103.15` ✅
- **Severity**: Medium (Revocation Check Bypass)
- **Date Verified**: 2026-10-11

## Vulnerability Details

If a certificate had more than one `distributionPoint`, only the first `distributionPoint` was considered against each CRL's `IssuingDistributionPoint`, causing subsequent distribution points to be ignored.

With `UnknownStatusPolicy::Allow`, this could lead to improper acceptance of revoked certificates.

## Current Status in RustIRC

### Dependency Chain

`rustls-webpki` is used via `rustls` 0.23.45 across all network-facing crates.

### Verification Evidence

1. **Cargo.lock Check**:
   `rustls-webpki` is locked at `0.103.15`, exceeding the minimum patched version `0.103.10`.
2. **Workspace Requirement**:
   `rustls = "0.23.45"` is enforced in `Cargo.toml`.
3. **Automated Test**:
   `tests/security_advisories_test.rs::test_rustls_webpki_root_store_and_server_names` validates webpki root certificates and ServerName parsing.
4. **Security Audit**:
   `cargo audit` passes cleanly with 0 vulnerabilities detected.

## Conclusion

RustIRC is secure against RUSTSEC-2026-0049. Issue #90 is fully resolved.
