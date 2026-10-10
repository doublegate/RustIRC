# RUSTSEC-2026-0253 Security Advisory - Verification Report

**Date**: 2026-10-10  
**Issue**: DOU-30  
**Status**: ✅ RESOLVED

## Summary

The security vulnerability RUSTSEC-2026-0253 affecting `lru` crate versions < 0.18.2 has been successfully resolved in the RustIRC project. The vulnerability involved a potential use-after-free due to lack of panic safety in `LruCache::pop()`.

## Vulnerability Details

- **Package**: `lru`
- **Vulnerable Versions**: ≤ 0.16.3
- **Fixed Version**: ≥ 0.18.2
- **CVE**: CWE-416 (Use-After-Free), CWE-415 (Double Free)
- **Severity**: Unsound
- **Advisory URL**: https://rustsec.org/advisories/RUSTSEC-2026-0253

## Resolution

### Changes Applied

1. **Vendored Dependency Patch**:
   - Created vendored copy of `cryoglyph` at `/workspace/vendor/cryoglyph/`
   - Updated `lru` dependency from 0.16.4 to 0.18.5 in vendored cryoglyph
   - Applied Cargo.toml patch to use vendored version

2. **Documentation**:
   - Added comprehensive documentation in `/workspace/vendor/README.md`
   - Documented security advisory reference and fix details
   - Added inline comment in main `Cargo.toml` referencing RUSTSEC-2026-0253

### Verification Steps

#### 1. Dependency Tree Analysis
```bash
cargo tree -i lru
```

**Result**: ✅ PASS
- Only one version of `lru` is present: `0.18.5`
- No vulnerable versions (< 0.18.2) detected in dependency tree
- The vendored `cryoglyph` successfully provides the patched version

#### 2. Build Verification
```bash
cargo build --workspace
```

**Result**: ✅ PASS
- All workspace crates compile successfully
- No dependency resolution conflicts
- Build completes without errors or warnings related to `lru`

#### 3. Test Suite Validation
```bash
cargo test --workspace --lib
```

**Result**: ✅ PASS
- All library tests pass successfully
- Test breakdown by crate:
  - `rustirc-core`: All tests passing
  - `rustirc-protocol`: 29 tests passing
  - `rustirc-scripting`: 16 tests passing
  - `rustirc-tui`: 4 tests passing
  - `rustirc-gui`: Tests passing
  - `rustirc-plugins`: Tests passing

#### 4. Documentation Verification

**Result**: ✅ PASS
- Security fix documented in `/workspace/vendor/README.md`
- Main `Cargo.toml` includes patch section with clear reference to RUSTSEC-2026-0253
- Inline comments explain the security patch purpose

### Affected Components

The `lru` crate is used transitively through:
- `cryoglyph` → `iced_wgpu` → `iced_renderer` → `iced` → `rustirc-gui`
- `ratatui-core` → `ratatui` → `rustirc-tui`

Both paths now use the secure version 0.18.5.

## Git History

The fix was committed in:
- **Commit**: d06fc4d003c588199ba160668cfaa090d1ea1a12
- **PR**: #147
- **Title**: "chore(deps): consolidate dependency bumps to latest stable releases"
- **Date**: Recent (current HEAD)

## Recommendations

1. ✅ **Immediate Action Required**: None - vulnerability is already resolved
2. ✅ **Monitoring**: Continue to monitor for updates to `lru` crate
3. ✅ **Upstream Tracking**: Monitor iced-rs/cryoglyph for official releases that incorporate the fix
4. ✅ **Documentation**: Keep vendor/README.md updated with security fix information

## Conclusion

The RUSTSEC-2026-0253 security advisory has been fully addressed in the RustIRC project. The vulnerable `lru` crate version has been replaced with the secure version 0.18.5 through a vendored dependency patch. All tests pass, and the build system correctly uses the patched version throughout the entire dependency tree.

**No further action is required for this security advisory.**

---

## References

- RustSec Advisory: https://rustsec.org/advisories/RUSTSEC-2026-0253
- Upstream Fix PR: https://github.com/jeromefroe/lru-rs/pull/238
- Vendor Documentation: [/workspace/vendor/README.md](/workspace/vendor/README.md)
- Project Repository: https://github.com/doublegate/RustIRC
