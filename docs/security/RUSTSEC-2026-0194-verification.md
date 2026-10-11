# Security Verification: RUSTSEC-2026-0194

## Summary

Verified that RustIRC is **NOT VULNERABLE** to RUSTSEC-2026-0194. The project uses `quick-xml` version 0.41.0, which eliminates the quadratic attribute checking vulnerability by replacing the linear duplicate scan with an $O(1)$ hash pre-filter.

- **GitHub Issues**: [#127](https://github.com/doublegate/RustIRC/issues/127), [#129](https://github.com/doublegate/RustIRC/issues/129)
- **Advisory**: RUSTSEC-2026-0194
- **Package**: `quick-xml`
- **Affected Versions**: `< 0.41.0`
- **Patched Versions**: `>= 0.41.0`
- **Current Version in RustIRC**: `0.41.0` ✅
- **Severity**: High (CVSS 7.5; CPU exhaustion / Denial of Service)
- **Date Verified**: 2026-10-11

## Vulnerability Details

In `quick-xml` versions prior to 0.41.0, `BytesStart::attributes()` returned an `Attributes` iterator that performed a linear scan against every attribute seen so far when verifying uniqueness (`with_checks(true)`). For a start tag with $N$ distinct attribute names, checking duplicate attributes required $O(N^2)$ byte comparisons.

A crafted start tag with many attributes could stall parsing threads for minutes to hours, denying service to the worker process.

## Resolution in quick-xml 0.41.0

In `quick-xml >= 0.41.0`:
- The duplicate check maintains a small linear scan for start tags with few attributes.
- Above a threshold, it switches to an $O(1)$ hash pre-filter, guaranteeing $O(N)$ total check time for the whole tag.
- Reported `AttrError::Duplicated` positions remain unchanged.

## Current Status in RustIRC

### Dependency Chain

`quick-xml` is pulled in transitively via the Wayland display stack:

```
quick-xml v0.41.0
└── wayland-scanner v0.31.11
    ├── smithay-client-toolkit v0.19.2 / v0.20.0
    │   ├── sctk-adwaita v0.10.1 -> winit -> iced_winit -> iced -> rustirc-gui -> rustirc
    │   └── wayland-client -> softbuffer, calloop, iced
    └── winit v0.30.13
```

### Verification Evidence

1. **Cargo.lock Check**:
   `quick-xml` is resolved to version `0.41.0`.
2. **Workspace Requirement**:
   `Cargo.toml` declares `quick-xml = "0.41.0"` in `[workspace.dependencies]` with an explicit advisory reference comment.
3. **Automated Test**:
   `test_quick_xml_duplicate_attribute_detection` checks duplicate rejection after 2,500 distinct attributes. It does not prove linear complexity. `test_security_advisory_dependency_floors` rejects quick-xml versions below 0.41.0, where the upstream algorithm was fixed. Wall-clock scaling assertions were removed because short, single-run timings are sensitive to scheduler noise. See the [upstream advisory](https://rustsec.org/advisories/RUSTSEC-2026-0194.html) for algorithm and benchmark evidence.
4. **Security Audit**:
   `cargo audit` returns 0 vulnerabilities.

## Conclusion

RustIRC is secure against RUSTSEC-2026-0194. Both Issue #127 and Issue #129 are fully resolved.
