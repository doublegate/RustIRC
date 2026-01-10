# Vendored Dependencies

This directory contains vendored copies of dependencies that have been patched for security or compatibility reasons.

## iced_glyphon

**Reason**: Security patch for RUSTSEC-2026-0002  
**Original version**: 0.6.0  
**Issue**: The original iced_glyphon 0.6.0 depends on lru 0.12.5, which contains a soundness bug where `IterMut` violates Stacked Borrows by invalidating internal pointers.

**Changes made**:
- Updated `lru` dependency from 0.12.1 to 0.16.3 in Cargo.toml

**Affected versions**: lru 0.9.0 - 0.16.2  
**Fixed version**: lru 0.16.3+

**Upstream tracking**:
- RustSec Advisory: https://rustsec.org/advisories/RUSTSEC-2026-0002
- lru-rs PR #224: https://github.com/jeromefroe/lru-rs/pull/224
- iced_glyphon issue: https://github.com/hecrj/glyphon (no newer version available yet)

**Future**: This patch can be removed once iced_glyphon releases a version that depends on lru 0.16.3 or higher, or when upgrading to iced 0.14+ which may use a different text rendering backend.
