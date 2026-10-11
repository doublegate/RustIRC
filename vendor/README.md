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

## cosmic-text

**Reason**: Partial mitigation for RUSTSEC-2026-0192 (unmaintained `ttf-parser`)  
**Original version**: 0.15.0  
**Changes made**:
- Updated `fontdb` dependency from 0.23.0 to 0.24.0 in `Cargo.toml`, removing the direct `fontdb -> ttf-parser` dependency path.
- Removed dangling `examples/*` workspace member glob from vendored `Cargo.toml`.

**Scope**:
- Eliminates the direct `cosmic-text -> fontdb -> ttf-parser` dependency edge.
- Note: Transitive `ttf-parser` remains in `Cargo.lock` via `sctk-adwaita -> ab_glyph -> owned_ttf_parser` as an accepted risk pending upstream updates in iced-rs/sctk-adwaita.

