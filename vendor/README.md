# Vendored Dependencies

This directory contains vendored copies of dependencies that have been patched for security or compatibility reasons.

## cryoglyph

**Reason**: Security patch for RUSTSEC-2026-0253  
**Original version**: 0.1.0  
**Issue**: The original cryoglyph 0.1.0 depends on lru 0.16.4, which contains a soundness bug (RUSTSEC-2026-0253) where `LruCache::pop()` lacks panic safety leading to potential use-after-free.

**Changes made**:
- Updated `lru` dependency from 0.16 to 0.18.5 in Cargo.toml

**Affected versions**: lru 0.16.4  
**Fixed version**: lru 0.18.5+

**Upstream tracking**:
- RustSec Advisory: https://rustsec.org/advisories/RUSTSEC-2026-0253
- iced-rs cryoglyph repository: https://github.com/iced-rs/cryoglyph

