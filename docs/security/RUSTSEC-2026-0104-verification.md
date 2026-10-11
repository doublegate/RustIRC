# Security Verification: RUSTSEC-2026-0104

## Summary

Verified that RustIRC is **NOT VULNERABLE** to RUSTSEC-2026-0104. The project uses `rustls-webpki` version 0.103.15, which resolves the reachable panic vulnerability during Certificate Revocation List (CRL) parsing.

- **GitHub Issue**: [#104](https://github.com/doublegate/RustIRC/issues/104)
- **Advisory**: RUSTSEC-2026-0104
- **Package**: `rustls-webpki`
- **Affected Versions**: `< 0.103.13` OR `>= 0.104.0-alpha.1, < 0.104.0-alpha.7`
- **Patched Versions**: `>= 0.103.13, < 0.104.0-alpha.1` OR `>= 0.104.0-alpha.7`
- **Current Version in RustIRC**: `0.103.15` ✅
- **Severity**: Medium (Reachable Panic / Denial of Service)
- **Date Verified**: 2026-10-11

## Vulnerability Details

A panic was reachable when parsing CRLs via `BorrowedCertRevocationList::from_der` or `OwnedCertRevocationList::from_der` due to mishandling a syntactically valid empty `BIT STRING` appearing in the `onlySomeReasons` element of an `IssuingDistributionPoint` CRL extension.

This panic was reachable prior to a CRL's signature being verified.

## Current Status in RustIRC

### Dependency Chain

`rustls-webpki` is pulled in via `rustls` (version 0.23.45) and `tokio-rustls`:

```text
rustirc -> rustirc-core -> rustls v0.23.45 -> rustls-webpki v0.103.15
rustirc -> rustirc-core -> tokio-rustls v0.26.6 -> rustls v0.23.45
```

The GUI, plugins, scripting and TUI crates reach these dependencies through
`rustirc-core`; they are not direct dependents of `rustls`. Verified with
`cargo tree --locked -i rustls-webpki`.

### Verification Evidence

1. **Cargo.lock Check**:
   `rustls-webpki` is locked at `0.103.15`, which exceeds the minimum patched threshold of `0.103.13`.
2. **Automated Test**:
   `test_rustls_webpki_root_store_and_server_names` is a configuration smoke test only; it does not validate chains, name constraints, or CRLs. `test_security_advisory_dependency_floors` rejects vulnerable resolved webpki versions. Advisory-specific certificate/CRL fixtures are not supplied by this PR.
3. **Security Audit**:
   `cargo audit` passes cleanly with 0 vulnerabilities detected.

## Conclusion

RustIRC is secure against RUSTSEC-2026-0104. Issue #104 is fully resolved.

## Application applicability

Not applicable to the current application CRL path: `crates/rustirc-core/src/connection.rs::create_tls_connector` configures trusted roots and no client authentication, with no CRL configuration or parsing. This does not mean RustIRC implements revocation checking. The dependency is patched independently. Invalid-DER smoke tests do not exercise the advisory-specific CRL extension or distribution-point matching.
