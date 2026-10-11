# RustIRC Security Documentation

This directory contains security-related documentation, including vulnerability assessments, security audit results, and mitigation strategies.

## Security Audit Process

RustIRC follows a proactive security approach:

1. **Regular Dependency Audits**: Using `cargo audit` to scan for known vulnerabilities
2. **Prompt Response**: Security advisories are reviewed and addressed promptly
3. **Documentation**: All security assessments are documented for transparency
4. **Version Management**: Dependencies are kept up-to-date with patched versions

## Running Security Audits

To perform a security audit on the project:

```bash
# Install cargo-audit if not already installed
cargo install cargo-audit

# Run the audit
cargo audit

# Run with strict warnings
cargo audit --deny warnings
```

## Security Verification Documents

| Advisory | GitHub Issue | Status | Document | Date |
|----------|--------------|--------|----------|------|
| RUSTSEC-2026-0044 | - | ✅ Not Vulnerable | [RUSTSEC-2026-0044-verification.md](./RUSTSEC-2026-0044-verification.md) | 2026-10-10 |
| RUSTSEC-2026-0048 | [#88](https://github.com/doublegate/RustIRC/issues/88) | ✅ Not Vulnerable | [SECURITY_VERIFICATION_RUSTSEC-2026-0048.md](../docs/security/SECURITY_VERIFICATION_RUSTSEC-2026-0048.md) | 2026-10-10 |
| RUSTSEC-2026-0049 | [#90](https://github.com/doublegate/RustIRC/issues/90) | ✅ Resolved | [RUSTSEC-2026-0049-resolution.md](../docs/security/RUSTSEC-2026-0049-resolution.md) | 2026-10-11 |
| RUSTSEC-2026-0097 | [#95](https://github.com/doublegate/RustIRC/issues/95) | ✅ Not Vulnerable | [RUSTSEC-2026-0097-verification.md](../docs/security/RUSTSEC-2026-0097-verification.md) | 2026-10-10 |
| RUSTSEC-2026-0098 | [#102](https://github.com/doublegate/RustIRC/issues/102) | ✅ Resolved | [RUSTSEC-2026-0098-resolution.md](../docs/security/RUSTSEC-2026-0098-resolution.md) | 2026-10-11 |
| RUSTSEC-2026-0099 | [#103](https://github.com/doublegate/RustIRC/issues/103) | ✅ Resolved | [RUSTSEC-2026-0099-resolution.md](../docs/security/RUSTSEC-2026-0099-resolution.md) | 2026-10-11 |
| RUSTSEC-2026-0104 | [#104](https://github.com/doublegate/RustIRC/issues/104) | ✅ Not Vulnerable | [RUSTSEC-2026-0104-verification.md](../docs/security/RUSTSEC-2026-0104-verification.md) | 2026-10-11 |
| RUSTSEC-2026-0186 | [#114](https://github.com/doublegate/RustIRC/issues/114) | ✅ Resolved | [SECURITY_AUDIT_RUSTSEC-2026-0186.md](../SECURITY_AUDIT_RUSTSEC-2026-0186.md) | 2026-10-10 |
| RUSTSEC-2026-0190 | [#132](https://github.com/doublegate/RustIRC/issues/132) | ✅ Not Vulnerable | [RUSTSEC-2026-0190-verification.md](../docs/security/RUSTSEC-2026-0190-verification.md) | 2026-10-11 |
| RUSTSEC-2026-0194 | [#127](https://github.com/doublegate/RustIRC/issues/127), [#129](https://github.com/doublegate/RustIRC/issues/129) | ✅ Not Vulnerable | [RUSTSEC-2026-0194-verification.md](../docs/security/RUSTSEC-2026-0194-verification.md) | 2026-10-11 |
| RUSTSEC-2026-0195 | [#130](https://github.com/doublegate/RustIRC/issues/130) | ✅ Not Vulnerable | [RUSTSEC-2026-0195-verification.md](../docs/security/RUSTSEC-2026-0195-verification.md) | 2026-10-11 |
| RUSTSEC-2026-0204 | [#136](https://github.com/doublegate/RustIRC/issues/136) | ✅ Not Vulnerable | [RUSTSEC-2026-0204-verification.md](../docs/security/RUSTSEC-2026-0204-verification.md) | 2026-10-11 |
| RUSTSEC-2026-0221 | [#137](https://github.com/doublegate/RustIRC/issues/137) | ✅ Not Vulnerable | [RUSTSEC-2026-0221-verification.md](../docs/security/RUSTSEC-2026-0221-verification.md) | 2026-10-11 |
| RUSTSEC-2026-0253 | [#140](https://github.com/doublegate/RustIRC/issues/140) | ✅ Resolved | [RUSTSEC-2026-0253-VERIFICATION.md](../RUSTSEC-2026-0253-VERIFICATION.md) | 2026-10-10 |

## Reporting Security Issues

If you discover a security vulnerability in RustIRC, please report it via:

- **GitHub Security Advisories**: Submit a private vulnerability report via the [GitHub Security tab](https://github.com/doublegate/RustIRC/security) per the project's [SECURITY.md](../SECURITY.md).

Please do NOT report security vulnerabilities through public GitHub issues.

## Security Best Practices

RustIRC follows these security best practices:

1. **TLS Supported**: Encrypted network communication is supported via `rustls` (plain TCP and TLS options are configurable in GUI and TUI connection dialogs)
2. **Memory Safety**: Leveraging Rust's ownership model and borrow checker
3. **Sandboxed Scripting**: Lua scripting environment with appropriate restrictions
4. **Input Validation**: All IRC messages are validated against malformed input
5. **Regular Updates**: Dependencies are kept current with security patches

## Acknowledgments

We appreciate the work of the Rust Security Response Working Group and the broader Rust security community for maintaining the RustSec Advisory Database.
