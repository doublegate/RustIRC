# Security Fix: RUSTSEC-2026-0002

## Summary

Fixed a soundness vulnerability in the `lru` crate (version 0.12.5) used as a transitive dependency through `iced_glyphon`.

## Vulnerability Details

- **Advisory**: RUSTSEC-2026-0002
- **Package**: `lru`
- **Affected Versions**: 0.9.0 to 0.16.2 (inclusive)
- **Patched Version**: 0.16.3+
- **Severity**: Unsound (memory safety issue)
- **Issue**: `IterMut` violates Stacked Borrows by invalidating internal pointer

### Technical Description

The `IterMut` iterator implementation in the vulnerable lru versions temporarily creates an exclusive reference (`&mut`) to the key when dereferencing the internal node pointer. This invalidates the shared pointer (`&`) held by the internal `HashMap`, violating Rust's Stacked Borrows rules and potentially causing undefined behavior.

## Dependency Chain

```
rustirc v0.3.8
└── rustirc-gui v0.3.8
    └── iced v0.13.1
        └── iced_wgpu v0.13.5
            └── iced_glyphon v0.6.0
                └── lru v0.12.5  ← VULNERABLE
```

## Solution Implemented

Since `iced_glyphon` v0.6.0 depends on `lru ^0.12.1` and there's no newer version of `iced_glyphon` available that uses the patched lru, we implemented a vendor patch:

1. **Downloaded** `iced_glyphon` v0.6.0 source code
2. **Modified** `Cargo.toml` to update lru dependency from `0.12.1` to `0.16.3`
3. **Vendored** the patched version in `vendor/iced_glyphon/`
4. **Applied** Cargo patch in workspace `Cargo.toml`:
   ```toml
   [patch.crates-io]
   iced_glyphon = { path = "vendor/iced_glyphon" }
   ```

## Verification

Before fix:
```
lru v0.12.5  ← Vulnerable
└── iced_glyphon v0.6.0
```

After fix:
```
lru v0.16.3  ← Patched
└── iced_glyphon v0.6.0 (vendored)
```

## Testing

- ✅ Clean build successful
- ✅ All tests passing
- ✅ Clippy clean (no warnings)
- ✅ No vulnerable lru versions in dependency tree

## Future Maintenance

This vendor patch can be removed when:
- `iced_glyphon` releases a version with lru 0.16.3+
- Upgrading to iced 0.14+ (which may use different text rendering)
- Switching to `cryoglyph` (iced-rs fork with updated dependencies)

## References

- RustSec Advisory: https://rustsec.org/advisories/RUSTSEC-2026-0002
- GitHub Advisory: https://github.com/advisories/GHSA-rhfx-m35p-ff5j
- lru-rs Fix PR: https://github.com/jeromefroe/lru-rs/pull/224
- Affected Package: https://crates.io/crates/lru/0.12.5
- Patched Package: https://crates.io/crates/lru/0.16.3

## Files Modified

- `Cargo.toml`: Added `[patch.crates-io]` section
- `Cargo.lock`: Updated lru dependency to 0.16.3
- `vendor/`: Added patched iced_glyphon source
- `vendor/README.md`: Documentation for vendored dependencies
