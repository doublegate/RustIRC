# Security Audit Report: RUSTSEC-2026-0186

## Issue Summary
**Advisory:** RUSTSEC-2026-0186  
**Package:** memmap2  
**Vulnerability:** Unchecked pointer offset  
**Severity:** Unsound  
**Date Reported:** 2026-06-20  
**Fixed Version:** 0.9.11

## Vulnerability Description
Affected versions of `memmap2` (≤ 0.9.10) did not perform enough validation on the `offset` and `len` parameters of:
- `Mmap::[unchecked_]advise_range()`
- `MmapMut::[unchecked_]advise_range()`
- `MmapMut::flush[_async]_range()`

This can cause undefined behavior due to invalid values being passed to `pointer::offset()` and `pointer::add()` when passing an out-of-bounds range to any of the affected functions.

## Audit Date
2026-10-10

## Status
**✅ NOT VULNERABLE - No action required**

## Findings

### Current Version in Use
The RustIRC project currently uses **memmap2 v0.9.11**, which is the fixed version that addresses this vulnerability.

### Dependency Chain
`memmap2` is a transitive dependency brought in through:
1. `fontdb` → `cosmic-text` (used by iced GUI framework)
2. `sctk-adwaita` → `winit` (window management)
3. `smithay-client-toolkit` (Wayland support)
4. `softbuffer` (rendering backend)

### Verification Steps Performed

1. **Dependency Analysis**
   - Checked `Cargo.lock` for memmap2 version
   - Confirmed only one version (0.9.11) is in use
   - Traced dependency chain using `cargo tree -i memmap2`

2. **Security Audit**
   - Installed and ran `cargo audit`
   - Result: No vulnerabilities related to memmap2
   - Only found maintenance warnings for unrelated crates (bincode, paste, ttf-parser)

3. **Build Verification**
   - Successfully built the project with `cargo build`
   - All workspace crates compiled without errors

4. **Test Verification**
   - Ran full test suite with `cargo test --workspace`
   - All 266 tests passed (233 unit + 33 integration tests)
   - All doctests passed (59 total across all crates)

### cargo audit Output
```
Scanning Cargo.lock for vulnerabilities (603 crate dependencies)

✅ No vulnerabilities found related to memmap2
⚠️  3 maintenance warnings (unrelated to this advisory)
```

## Recommendations
1. **No immediate action required** - The project is already using the secure version
2. **Continue monitoring** - Keep dependencies up to date using `cargo update`
3. **Regular audits** - Run `cargo audit` periodically to catch new advisories
4. **Dependency updates** - Consider updating unmaintained dependencies when alternatives are available

## References
- Advisory: https://rustsec.org/advisories/RUSTSEC-2026-0186.html
- GitHub Issue: https://github.com/RazrFalcon/memmap2-rs/issues/169
- Fix Commit: https://github.com/RazrFalcon/memmap2-rs/pull/170/changes/cee7cf03a9ee095982a3c37b7aac8e3f68f1a00c

## Conclusion
The RustIRC project is **not affected** by RUSTSEC-2026-0186. The project is using memmap2 v0.9.11, which includes the fix for this vulnerability. All build and test verifications pass successfully.

---
**Auditor:** Cloud Agent (Cursor)  
**Date:** 2026-10-10  
**Linear Issue:** DOU-21
