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

| Advisory | Status | Document | Date |
|----------|--------|----------|------|
| RUSTSEC-2026-0044 | ✅ Not Vulnerable | [RUSTSEC-2026-0044-verification.md](./RUSTSEC-2026-0044-verification.md) | 2026-10-10 |

## Reporting Security Issues

If you discover a security vulnerability in RustIRC, please report it to:

- **Email**: security@example.com (to be configured)
- **GitHub Security Advisories**: Use the [GitHub Security tab](https://github.com/doublegate/RustIRC/security)

Please do NOT report security vulnerabilities through public GitHub issues.

## Security Best Practices

RustIRC follows these security best practices:

1. **TLS by Default**: All network communication uses TLS via `rustls`
2. **Memory Safety**: Leveraging Rust's ownership model and borrow checker
3. **Sandboxed Scripting**: Lua scripting environment with appropriate restrictions
4. **Input Validation**: All IRC messages are validated against malformed input
5. **Regular Updates**: Dependencies are kept current with security patches

## Acknowledgments

We appreciate the work of the Rust Security Response Working Group and the broader Rust security community for maintaining the RustSec Advisory Database.
