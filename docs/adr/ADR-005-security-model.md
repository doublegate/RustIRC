# ADR-005: Security Model

## Status
Accepted

## Context
IRC clients handle untrusted data from networks and users. Security vulnerabilities could lead to remote code execution, data theft, or privacy violations. A comprehensive security model is essential.

## Decision
We will implement **defense-in-depth security** with multiple layers of protection.

## Security Layers

### 1. Network Security
- **TLS by Default**: Prefer encrypted connections
- **Certificate Validation**: Verify server certificates
- **Certificate Pinning**: Optional per-server pinning
- **SASL Authentication**: Support secure authentication
- **Connection Limits**: Prevent resource exhaustion

### 2. Input Validation
- **Message Parsing**: Strict RFC compliance with bounds checking
- **UTF-8 Validation**: Reject invalid sequences
- **Length Limits**: Enforce 512-byte message limit
- **Rate Limiting**: Prevent flood attacks
- **Command Injection Prevention**: Sanitize all inputs

### 3. Script Sandboxing
- **Memory Limits**: Prevent memory exhaustion
- **CPU Limits**: Prevent infinite loops
- **No File Access**: Unless explicitly granted
- **No Network Access**: Unless explicitly granted
- **No Process Spawning**: Unless explicitly granted

### 4. Data Protection
- **Password Storage**: OS keychain integration
- **Log Sanitization**: Remove passwords from logs
- **Secure Wipe**: Overwrite sensitive memory
- **Permission Model**: Granular access control

### 5. Plugin Security
- **Signature Verification**: For distributed plugins
- **Capability Model**: Explicit permission grants
- **Resource Quotas**: CPU, memory, I/O limits
- **Isolation**: Separate process for untrusted plugins

## Threat Model

### External Threats
1. **Malicious Servers**: Sending crafted messages
2. **Network Attacks**: MITM, eavesdropping
3. **Flood Attacks**: Resource exhaustion
4. **Social Engineering**: Phishing via messages

### Internal Threats
1. **Malicious Scripts**: Attempting privilege escalation
2. **Vulnerable Plugins**: Introducing security holes
3. **User Errors**: Accidental credential exposure

## Implementation Guidelines

### Secure Coding Practices
- No unsafe code in message parsing
- Bounds checking on all buffers
- Input validation at boundaries
- Fail-safe defaults
- Least privilege principle

### Cryptography
- Use rustls for TLS
- Use ring for crypto primitives
- No custom crypto implementations
- Secure random number generation

### Error Handling
- Don't leak sensitive info in errors
- Log security events
- Fail securely (deny by default)
- Rate limit error responses

## Security Features

### DCC Security
- Confirm file transfers
- Scan for malware (optional)
- Sandbox downloaded files
- IP address masking
- Resume verification

### URL Preview
- Sandbox rendering
- Block tracking pixels
- Warn about suspicious URLs
- Certificate verification

### Anti-Spam
- Bayesian filtering (optional)
- Regex filters
- Rate limiting
- Ignore lists
- CTCP flood protection

## Audit and Monitoring

### Security Logging
- Authentication attempts
- Permission changes
- Plugin installations
- Suspicious patterns
- Error conditions

### Update Security
- Signed updates
- TLS for downloads
- Rollback capability
- Changelog verification

## Validation
Security testing demonstrated:
- Resistance to fuzzing attacks
- Proper TLS verification
- Effective sandboxing
- No memory leaks under stress