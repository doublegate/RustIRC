# RUSTSEC-2026-0044 Security Verification

## Issue Details

**Advisory:** RUSTSEC-2026-0044  
**Title:** AWS-LC X.509 Name Constraints Bypass via Wildcard/Unicode CN  
**Package:** `aws-lc-sys`  
**Vulnerable Version:** `0.38.0`  
**Patched Versions:** `>=0.39.0`  
**Date Reported:** 2026-03-19  
**Date Verified:** 2026-10-10

## Vulnerability Summary

A logic error in CN (Common Name) validation allows certificates with wildcard or raw UTF-8 Unicode CN values to bypass name constraints enforcement. The `cn2dnsid` function does not recognize these CN patterns as valid DNS identifiers, causing `NAME_CONSTRAINTS_check_CN` to skip validation. However, `X509_check_host` accepts these CN values when no dNSName SAN is present, allowing certificates to bypass name constraints while still being used for hostname verification.

## Verification Results

**Status:** ✅ NOT VULNERABLE

### Current Dependency Version

```
aws-lc-sys v0.45.0
```

### Dependency Chain

```
aws-lc-sys v0.45.0
└── aws-lc-rs v1.18.1
    ├── rustls v0.23.45
    │   ├── rustirc-core v0.4.2
    │   └── tokio-rustls v0.26.6
    └── rustls-webpki v0.103.15
        └── rustls v0.23.45
```

### Verification Method

1. **Direct Cargo.lock Inspection:**
   - Confirmed `aws-lc-sys` version is `0.45.0`
   - Version `0.45.0` is significantly higher than the patched version requirement `>=0.39.0`

2. **Dependency Tree Analysis:**
   - Used `cargo tree -i aws-lc-sys` to trace the complete dependency chain
   - Confirmed it's pulled in through `rustls` via `aws-lc-rs`

3. **Security Audit:**
   - Ran `cargo audit` (version latest)
   - **Result:** No vulnerabilities found related to `aws-lc-sys`
   - Only 3 unrelated "unmaintained" warnings for other crates (bincode, paste, ttf-parser)

### Audit Output

```
Scanning Cargo.lock for vulnerabilities (603 crate dependencies)
warning: 3 allowed warnings found
```

**No RUSTSEC-2026-0044 advisory detected in the codebase.**

## Conclusion

The RustIRC project is **NOT affected** by RUSTSEC-2026-0044. The project uses `aws-lc-sys v0.45.0`, which is well above the minimum patched version of `0.39.0` and therefore includes the fix for this vulnerability.

### Recommendation

✅ **No action required.** The dependency is already at a safe version.

### Verification Performed By

- Cursor Cloud Agent
- Date: 2026-10-10

### References

- [RUSTSEC-2026-0044 Advisory](https://rustsec.org/advisories/RUSTSEC-2026-0044.html)
- [aws-lc-sys Crate](https://crates.io/crates/aws-lc-sys)
