# Security Advisory Resolution: RUSTSEC-2026-0098

## Summary

**Status**: ✅ **RESOLVED** - No action required

**Advisory**: RUSTSEC-2026-0098  
**Title**: Name constraints for URI names were incorrectly accepted  
**Package**: `rustls-webpki`  
**Date Reported**: 2026-04-14  
**Date Verified**: 2026-10-10

## Vulnerability Details

### Affected Versions
- Vulnerable: `rustls-webpki < 0.103.12`
- Patched: `>=0.103.12, <0.104.0-alpha.1, >=0.104.0-alpha.6`

### Description
Name constraints for URI names were ignored and therefore accepted in vulnerable versions. Since name constraints are restrictions on otherwise properly-issued certificates, this bug is reachable only after signature verification and requires certificate misissuance to exploit.

### Security Impact
- **Severity**: Medium
- **Exploitability**: Requires misissuance to exploit
- **Scope**: Limited to TLS certificate validation

## Current Status in RustIRC

### Dependency Analysis

**Current Version in Use**: `rustls-webpki v0.103.15`

```
rustls-webpki v0.103.15
└── rustls v0.23.45
    ├── rustirc-core v0.4.2
    ├── rustirc-gui v0.4.2
    ├── rustirc-plugins v0.4.2
    ├── rustirc-scripting v0.4.2
    ├── rustirc-tui v0.4.2
    └── tokio-rustls v0.26.6
```

**Dependency Path**: `rustls-webpki` is a transitive dependency through:
1. `rustls = "0.23"` (workspace dependency)
2. `tokio-rustls = "0.26"` (workspace dependency)

### Verification Results

#### 1. Version Check
```bash
$ grep -A 5 "name = \"rustls-webpki\"" Cargo.lock
[[package]]
name = "rustls-webpki"
version = "0.103.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f3c3cf1d8b1e7d4927e2d154c3fcb02979afb9939629c62cd9048d4f07b60ac2"
```

**Result**: ✅ Version 0.103.15 is well above the minimum patched version (0.103.12)

#### 2. Security Audit
```bash
$ cargo audit
    Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
      Loaded 1296 security advisories
    Scanning Cargo.lock for vulnerabilities (603 crate dependencies)

warning: 3 allowed warnings found (unrelated to this advisory)
```

**Result**: ✅ No security vulnerabilities detected, RUSTSEC-2026-0098 not flagged

#### 3. Build Verification
```bash
$ cargo build --quiet
```

**Result**: ✅ Build successful with no errors

#### 4. Test Suite
```bash
$ cargo test --quiet
```

**Result**: ✅ All 33 tests passed across all workspace crates

## Conclusion

The RustIRC project is **NOT AFFECTED** by RUSTSEC-2026-0098. The project currently uses `rustls-webpki v0.103.15`, which is a patched version that includes the fix for this vulnerability.

### Timeline
- **2026-04-14**: Vulnerability disclosed
- **2026-10-10**: Verification completed - confirmed patched version in use

### Recommendations
1. ✅ No immediate action required
2. ✅ Continue monitoring `rustls` and `tokio-rustls` updates
3. ✅ Run `cargo audit` regularly as part of CI/CD pipeline
4. ✅ Keep dependency updates timely to receive security patches

## References

- [RUSTSEC-2026-0098 Advisory](https://rustsec.org/advisories/RUSTSEC-2026-0098.html)
- [GitHub Security Advisory GHSA-965h-392x-2mh5](https://github.com/rustls/webpki/security/advisories/GHSA-965h-392x-2mh5)
- [rustls-webpki Repository](https://github.com/rustls/webpki)

## Verified By

- **Date**: 2026-10-10
- **Method**: Automated dependency analysis and security audit
- **Tools**: `cargo tree`, `cargo audit`, `cargo test`
