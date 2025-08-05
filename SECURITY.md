# Security Policy

## Supported Versions

As RustIRC is currently in development, security updates will be provided for:

| Version | Supported          |
| ------- | ------------------ |
| main    | :white_check_mark: |
| develop | :white_check_mark: |
| < 1.0   | :x:                |

Once we reach 1.0, we will maintain security updates for the current major version and the previous major version.

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security vulnerability in RustIRC, please follow these steps:

### 1. Do NOT Create a Public Issue

Security vulnerabilities should not be reported through public GitHub issues.

### 2. Report Privately

Send an email to: security@rustirc.org (once established)

Or use GitHub's private vulnerability reporting:
1. Go to the Security tab in the repository
2. Click "Report a vulnerability"
3. Fill out the form with details

### 3. Include Details

Please include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)
- Your contact information

### 4. Response Time

- We will acknowledge receipt within 48 hours
- We will provide an initial assessment within 1 week
- We will work on a fix and coordinate disclosure

## Security Best Practices for RustIRC

### For Users

1. **Keep RustIRC Updated** - Always use the latest version
2. **Use TLS/SSL** - Always connect to IRC servers using TLS
3. **Strong Passwords** - Use strong, unique passwords for IRC services
4. **Script Safety** - Only install scripts from trusted sources
5. **Plugin Verification** - Verify plugin signatures before installation

### For Developers

1. **Input Validation** - Always validate and sanitize user input
2. **Memory Safety** - Leverage Rust's memory safety features
3. **Dependency Audit** - Regularly audit dependencies with `cargo audit`
4. **Secure Defaults** - Default to secure configurations
5. **Principle of Least Privilege** - Scripts and plugins run with minimal permissions

## Security Features in RustIRC

- **Sandboxed Scripting** - Lua and Python scripts run in restricted environments
- **TLS by Default** - Secure connections preferred
- **SASL Authentication** - Support for secure authentication methods
- **Input Sanitization** - Protection against IRC protocol injection
- **Resource Limits** - Prevent resource exhaustion attacks
- **Secure Storage** - Credentials stored securely using platform keychains

## Vulnerability Disclosure Policy

1. **Private Disclosure** - Vulnerabilities are disclosed privately to affected parties
2. **Patch Development** - We develop patches in private repositories
3. **Coordinated Release** - Patches are released simultaneously with disclosure
4. **Credit** - Security researchers are credited (unless they prefer anonymity)
5. **CVE Assignment** - Critical vulnerabilities receive CVE identifiers

## Security Audit

RustIRC undergoes regular security reviews:
- Automated scanning with `cargo audit`
- Dependency review with Dependabot
- Code review for all security-related changes
- Planned third-party audit before 1.0 release

## Contact

- Security Email: security@rustirc.org (to be established)
- GitHub Security Advisories: Via repository Security tab
- General Inquiries: Use GitHub Discussions

Thank you for helping keep RustIRC secure!