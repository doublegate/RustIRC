# SASL Authentication Specification

## Overview

Simple Authentication and Security Layer (SASL) provides a framework for authentication and optional security layer negotiation in IRC. This document specifies RustIRC's implementation of SASL authentication mechanisms, focusing on security best practices and IRCv3 compliance.

## SASL in IRC Context

### IRCv3 SASL Extension

SASL authentication occurs during connection registration, before the NICK/USER commands:

```
Client: CAP LS 302
Server: CAP * LS :sasl=PLAIN,EXTERNAL,SCRAM-SHA-256,SCRAM-SHA-512
Client: CAP REQ :sasl
Server: CAP * ACK :sasl
Client: AUTHENTICATE PLAIN
Server: AUTHENTICATE +
Client: AUTHENTICATE <base64-encoded-credentials>
Server: 900 nick!user@host nick :You are now logged in as username
Server: 903 nick :SASL authentication successful
Client: CAP END
```

### Authentication Flow

```rust
pub struct SaslAuthenticator {
    mechanisms: HashMap<String, Box<dyn SaslMechanism>>,
    current_mechanism: Option<String>,
    state: AuthState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    Idle,
    InProgress,
    Completed,
    Failed(String),
}

impl SaslAuthenticator {
    pub async fn authenticate(
        &mut self,
        mechanism: &str,
        credentials: SaslCredentials
    ) -> Result<()> {
        // Select mechanism
        let mech = self.mechanisms.get(mechanism)
            .ok_or(SaslError::UnsupportedMechanism)?;
        
        self.current_mechanism = Some(mechanism.to_string());
        self.state = AuthState::InProgress;
        
        // Start authentication
        let initial_response = mech.initial_response(&credentials)?;
        
        // Send AUTHENTICATE command
        if initial_response.is_empty() {
            send_command("AUTHENTICATE", &[mechanism]).await?;
        } else {
            let encoded = base64_encode(&initial_response);
            send_authenticate_data(&encoded).await?;
        }
        
        Ok(())
    }
}
```

## Supported Mechanisms

### PLAIN Mechanism

The simplest mechanism, sending credentials in plaintext (must use TLS):

```rust
pub struct PlainMechanism;

impl SaslMechanism for PlainMechanism {
    fn initial_response(&self, creds: &SaslCredentials) -> Result<Vec<u8>> {
        // Format: authzid\0authcid\0password
        let authzid = creds.authzid.as_deref().unwrap_or("");
        let authcid = &creds.username;
        let password = &creds.password;
        
        let response = format!("{}\0{}\0{}", authzid, authcid, password);
        Ok(response.into_bytes())
    }
    
    fn process_challenge(&mut self, _challenge: &[u8]) -> Result<Vec<u8>> {
        // PLAIN doesn't use challenges
        Err(SaslError::UnexpectedChallenge)
    }
}

// Security check
impl PlainMechanism {
    pub fn validate_security(connection: &Connection) -> Result<()> {
        if !connection.is_tls() {
            return Err(SaslError::InsecureConnection(
                "PLAIN requires TLS connection".to_string()
            ));
        }
        Ok(())
    }
}
```

### EXTERNAL Mechanism

Uses external authentication (typically TLS client certificates):

```rust
pub struct ExternalMechanism;

impl SaslMechanism for ExternalMechanism {
    fn initial_response(&self, creds: &SaslCredentials) -> Result<Vec<u8>> {
        // Send authzid or empty string
        Ok(creds.authzid.as_deref().unwrap_or("").as_bytes().to_vec())
    }
    
    fn process_challenge(&mut self, _challenge: &[u8]) -> Result<Vec<u8>> {
        Err(SaslError::UnexpectedChallenge)
    }
}

impl ExternalMechanism {
    pub fn validate_certificate(connection: &Connection) -> Result<String> {
        let cert = connection.peer_certificate()
            .ok_or(SaslError::NoCertificate)?;
        
        // Extract identity from certificate
        let subject = cert.subject();
        let cn = subject.common_name()
            .ok_or(SaslError::InvalidCertificate)?;
        
        // Verify certificate is valid
        cert.verify_hostname(&connection.hostname())?;
        
        Ok(cn.to_string())
    }
}
```

### SCRAM-SHA-256 Mechanism

Salted Challenge Response Authentication Mechanism with SHA-256:

```rust
pub struct ScramSha256Mechanism {
    state: ScramState,
    client_nonce: String,
    server_nonce: Option<String>,
    salt: Option<Vec<u8>>,
    iterations: Option<u32>,
    auth_message: Vec<u8>,
}

#[derive(Debug)]
enum ScramState {
    Initial,
    ClientFirst,
    ServerFirst,
    ClientFinal,
    Complete,
}

impl SaslMechanism for ScramSha256Mechanism {
    fn initial_response(&self, creds: &SaslCredentials) -> Result<Vec<u8>> {
        // Generate client nonce
        let client_nonce = generate_nonce();
        
        // Build client-first message
        let gs2_header = "n,,"; // No channel binding
        let username = saslprep(&creds.username)?;
        let client_first_bare = format!("n={},r={}", username, client_nonce);
        
        let client_first = format!("{}{}", gs2_header, client_first_bare);
        
        self.state = ScramState::ClientFirst;
        self.client_nonce = client_nonce;
        
        Ok(client_first.into_bytes())
    }
    
    fn process_challenge(&mut self, challenge: &[u8]) -> Result<Vec<u8>> {
        match self.state {
            ScramState::ClientFirst => self.handle_server_first(challenge),
            _ => Err(SaslError::InvalidState),
        }
    }
}

impl ScramSha256Mechanism {
    fn handle_server_first(&mut self, challenge: &[u8]) -> Result<Vec<u8>> {
        let server_first = std::str::from_utf8(challenge)?;
        let params = parse_scram_message(server_first)?;
        
        // Extract parameters
        let nonce = params.get("r").ok_or(SaslError::MissingParameter)?;
        let salt = base64_decode(params.get("s").ok_or(SaslError::MissingParameter)?)?;
        let iterations = params.get("i")
            .ok_or(SaslError::MissingParameter)?
            .parse::<u32>()?;
        
        // Verify nonce starts with client nonce
        if !nonce.starts_with(&self.client_nonce) {
            return Err(SaslError::InvalidNonce);
        }
        
        self.server_nonce = Some(nonce.to_string());
        self.salt = Some(salt);
        self.iterations = Some(iterations);
        
        // Generate client final message
        let client_final = self.generate_client_final()?;
        
        self.state = ScramState::ClientFinal;
        Ok(client_final)
    }
    
    fn generate_client_final(&mut self) -> Result<Vec<u8>> {
        let channel_binding = "c=biws"; // No channel binding
        let nonce = format!("r={}", self.server_nonce.as_ref().unwrap());
        
        let client_final_without_proof = format!("{},{}", channel_binding, nonce);
        
        // Calculate proof
        let salted_password = pbkdf2_hmac_sha256(
            &self.password,
            &self.salt.as_ref().unwrap(),
            self.iterations.unwrap()
        );
        
        let client_key = hmac_sha256(&salted_password, b"Client Key");
        let stored_key = sha256(&client_key);
        
        // Build auth message
        let auth_message = format!("{},{},{}",
            self.client_first_bare,
            self.server_first,
            client_final_without_proof
        );
        
        let client_signature = hmac_sha256(&stored_key, auth_message.as_bytes());
        let client_proof = xor(&client_key, &client_signature);
        
        let client_final = format!("{},p={}",
            client_final_without_proof,
            base64_encode(&client_proof)
        );
        
        self.auth_message = auth_message.into_bytes();
        
        Ok(client_final.into_bytes())
    }
}
```

### SCRAM Implementation Details

```rust
/// PBKDF2-HMAC-SHA256 implementation
fn pbkdf2_hmac_sha256(password: &str, salt: &[u8], iterations: u32) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    
    type HmacSha256 = Hmac<Sha256>;
    
    let mut result = vec![0u8; 32];
    pbkdf2::pbkdf2::<HmacSha256>(
        password.as_bytes(),
        salt,
        iterations,
        &mut result
    );
    result
}

/// SASLprep normalization
fn saslprep(input: &str) -> Result<String> {
    // Implement RFC 4013 SASLprep
    // 1. Map characters (NFKC normalization)
    // 2. Normalize (NFC)
    // 3. Prohibit certain characters
    // 4. Check bidirectional characters
    
    use unicode_normalization::UnicodeNormalization;
    
    let normalized = input.nfkc().collect::<String>();
    
    // Check for prohibited characters
    for ch in normalized.chars() {
        if is_prohibited_char(ch) {
            return Err(SaslError::ProhibitedCharacter(ch));
        }
    }
    
    Ok(normalized)
}

fn is_prohibited_char(ch: char) -> bool {
    matches!(ch,
        '\u{0000}'..='\u{001F}' | // Control characters
        '\u{007F}' |              // Delete
        '\u{0080}'..='\u{009F}' | // Control characters
        '\u{2028}' | '\u{2029}' | // Line/paragraph separators
        '\u{FFFE}' | '\u{FFFF}'   // Non-characters
    )
}
```

## Security Considerations

### Connection Security

```rust
pub struct SaslSecurityConfig {
    /// Require TLS for PLAIN mechanism
    pub require_tls_for_plain: bool,
    
    /// Minimum TLS version
    pub min_tls_version: TlsVersion,
    
    /// Allowed mechanisms
    pub allowed_mechanisms: HashSet<String>,
    
    /// Certificate validation
    pub verify_certificates: bool,
    
    /// SCRAM iteration minimum
    pub min_scram_iterations: u32,
}

impl Default for SaslSecurityConfig {
    fn default() -> Self {
        Self {
            require_tls_for_plain: true,
            min_tls_version: TlsVersion::Tls12,
            allowed_mechanisms: ["EXTERNAL", "SCRAM-SHA-256", "SCRAM-SHA-512"]
                .iter().map(|s| s.to_string()).collect(),
            verify_certificates: true,
            min_scram_iterations: 4096,
        }
    }
}
```

### Credential Storage

```rust
use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SaslCredentials {
    pub username: String,
    pub password: String,
    pub authzid: Option<String>,
}

/// Secure credential storage
pub struct CredentialStore {
    keyring: Keyring,
}

impl CredentialStore {
    pub async fn store_credentials(
        &mut self,
        server: &str,
        creds: SaslCredentials
    ) -> Result<()> {
        // Encrypt credentials
        let encrypted = self.encrypt_credentials(&creds)?;
        
        // Store in system keyring
        self.keyring.set_password(
            "rustirc",
            &format!("sasl:{}", server),
            &base64_encode(&encrypted)
        )?;
        
        Ok(())
    }
    
    pub async fn retrieve_credentials(&self, server: &str) -> Result<SaslCredentials> {
        // Retrieve from keyring
        let encrypted = self.keyring.get_password(
            "rustirc",
            &format!("sasl:{}", server)
        )?;
        
        // Decrypt
        let decrypted = self.decrypt_credentials(&base64_decode(&encrypted)?)?;
        
        Ok(decrypted)
    }
}
```

### Mechanism Selection

```rust
pub struct MechanismSelector {
    config: SaslSecurityConfig,
}

impl MechanismSelector {
    pub fn select_mechanism(
        &self,
        available: &[String],
        connection: &Connection
    ) -> Result<String> {
        // Filter by security requirements
        let secure_mechs: Vec<&String> = available.iter()
            .filter(|m| self.is_mechanism_secure(m, connection))
            .collect();
        
        // Preference order
        const PREFERENCE: &[&str] = &[
            "SCRAM-SHA-512",
            "SCRAM-SHA-256", 
            "EXTERNAL",
            "PLAIN",
        ];
        
        for pref in PREFERENCE {
            if secure_mechs.iter().any(|m| m == pref) {
                return Ok(pref.to_string());
            }
        }
        
        Err(SaslError::NoSuitableMechanism)
    }
    
    fn is_mechanism_secure(&self, mechanism: &str, connection: &Connection) -> bool {
        // Check if mechanism is allowed
        if !self.config.allowed_mechanisms.contains(mechanism) {
            return false;
        }
        
        // PLAIN requires TLS
        if mechanism == "PLAIN" && self.config.require_tls_for_plain {
            return connection.is_tls();
        }
        
        // EXTERNAL requires client certificate
        if mechanism == "EXTERNAL" {
            return connection.has_client_certificate();
        }
        
        true
    }
}
```

## Error Handling

### SASL Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum SaslError {
    #[error("Unsupported mechanism: {0}")]
    UnsupportedMechanism(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Insecure connection: {0}")]
    InsecureConnection(String),
    
    #[error("No client certificate available")]
    NoCertificate,
    
    #[error("Invalid certificate")]
    InvalidCertificate,
    
    #[error("SASL negotiation timeout")]
    Timeout,
    
    #[error("Invalid SASL state")]
    InvalidState,
    
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),
    
    #[error("Invalid server nonce")]
    InvalidNonce,
    
    #[error("Prohibited character in username: {0}")]
    ProhibitedCharacter(char),
    
    #[error("SCRAM iteration count too low: {0}")]
    WeakIterationCount(u32),
}
```

### Numeric Replies

```rust
/// SASL-related IRC numerics
pub mod sasl_numerics {
    pub const RPL_LOGGEDIN: u16 = 900;
    pub const RPL_LOGGEDOUT: u16 = 901;
    pub const ERR_NICKLOCKED: u16 = 902;
    pub const RPL_SASLSUCCESS: u16 = 903;
    pub const ERR_SASLFAIL: u16 = 904;
    pub const ERR_SASLTOOLONG: u16 = 905;
    pub const ERR_SASLABORTED: u16 = 906;
    pub const ERR_SASLALREADY: u16 = 907;
    pub const RPL_SASLMECHS: u16 = 908;
}
```

## Implementation Example

### Complete SASL Flow

```rust
pub async fn perform_sasl_auth(
    connection: &mut Connection,
    config: &ClientConfig
) -> Result<()> {
    // Request SASL capability
    connection.send("CAP REQ :sasl").await?;
    
    // Wait for ACK
    let ack = connection.receive().await?;
    if !ack.params.contains(&"sasl".to_string()) {
        return Err(SaslError::NotSupported);
    }
    
    // Get available mechanisms
    let mechanisms = connection.get_sasl_mechanisms().await?;
    
    // Select best mechanism
    let selector = MechanismSelector::new(&config.sasl_config);
    let mechanism = selector.select_mechanism(&mechanisms, connection)?;
    
    // Get credentials
    let credentials = if mechanism == "EXTERNAL" {
        SaslCredentials {
            username: String::new(),
            password: String::new(),
            authzid: config.sasl_authzid.clone(),
        }
    } else {
        config.get_credentials().await?
    };
    
    // Create authenticator
    let mut auth = SaslAuthenticator::new();
    auth.register_mechanism("PLAIN", Box::new(PlainMechanism));
    auth.register_mechanism("EXTERNAL", Box::new(ExternalMechanism));
    auth.register_mechanism("SCRAM-SHA-256", Box::new(ScramSha256Mechanism::new()));
    
    // Start authentication
    auth.authenticate(&mechanism, credentials).await?;
    
    // Handle server responses
    loop {
        let msg = connection.receive().await?;
        
        match msg.command.parse::<u16>() {
            Ok(numerics::RPL_SASLSUCCESS) => {
                info!("SASL authentication successful");
                break;
            }
            Ok(numerics::ERR_SASLFAIL) => {
                return Err(SaslError::AuthenticationFailed(
                    msg.params.get(1).cloned().unwrap_or_default()
                ));
            }
            _ if msg.command == "AUTHENTICATE" => {
                let response = if msg.params[0] == "+" {
                    auth.continue_auth(None).await?
                } else {
                    let challenge = base64_decode(&msg.params[0])?;
                    auth.continue_auth(Some(&challenge)).await?
                };
                
                send_authenticate_data(connection, &response).await?;
            }
            _ => {
                warn!("Unexpected message during SASL: {:?}", msg);
            }
        }
    }
    
    Ok(())
}

/// Send AUTHENTICATE data, handling long messages
async fn send_authenticate_data(connection: &mut Connection, data: &[u8]) -> Result<()> {
    let encoded = base64_encode(data);
    
    // IRC line length limit requires chunking
    const CHUNK_SIZE: usize = 400;
    
    if encoded.is_empty() {
        connection.send("AUTHENTICATE +").await?;
    } else {
        for chunk in encoded.as_bytes().chunks(CHUNK_SIZE) {
            let chunk_str = std::str::from_utf8(chunk)?;
            connection.send(&format!("AUTHENTICATE {}", chunk_str)).await?;
        }
        
        // Send + to indicate end if multiple chunks
        if encoded.len() > CHUNK_SIZE {
            connection.send("AUTHENTICATE +").await?;
        }
    }
    
    Ok(())
}
```

## Testing SASL

### Unit Tests

```rust
#[cfg(test)]
mod sasl_tests {
    use super::*;
    
    #[test]
    fn test_plain_mechanism() {
        let mech = PlainMechanism;
        let creds = SaslCredentials {
            username: "user".to_string(),
            password: "pass".to_string(),
            authzid: None,
        };
        
        let response = mech.initial_response(&creds).unwrap();
        assert_eq!(response, b"\0user\0pass");
    }
    
    #[test]
    fn test_scram_client_first() {
        let mut mech = ScramSha256Mechanism::new();
        let creds = SaslCredentials {
            username: "user".to_string(),
            password: "pass".to_string(),
            authzid: None,
        };
        
        let response = mech.initial_response(&creds).unwrap();
        let response_str = std::str::from_utf8(&response).unwrap();
        
        assert!(response_str.starts_with("n,,n=user,r="));
    }
    
    #[test]
    fn test_saslprep() {
        // Test normalization
        assert_eq!(saslprep("user").unwrap(), "user");
        
        // Test prohibited characters
        assert!(saslprep("user\0").is_err());
        assert!(saslprep("user\u{0001}").is_err());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_sasl_negotiation() {
    let mut server = MockIrcServer::new();
    server.expect_capability_negotiation("sasl");
    server.expect_authenticate("PLAIN");
    server.send_authenticate_challenge("+");
    server.expect_authenticate_response("AHVzZXIAcGFzcw=="); // \0user\0pass
    server.send_sasl_success();
    
    let config = ClientConfig {
        sasl: Some(SaslConfig {
            username: "user".to_string(),
            password: "pass".to_string(),
            mechanisms: vec!["PLAIN".to_string()],
        }),
        ..Default::default()
    };
    
    let mut client = Client::new();
    client.connect_to_mock(server, config).await.unwrap();
}
```

## Best Practices

1. **Always prefer SCRAM mechanisms** over PLAIN when available
2. **Use TLS** for all SASL authentication
3. **Store credentials securely** using system keyrings
4. **Implement mechanism fallback** for compatibility
5. **Validate server certificates** to prevent MITM
6. **Handle authentication failures gracefully** with clear user feedback
7. **Support reauthentication** for long-lived connections
8. **Zero credentials from memory** after use
9. **Log authentication attempts** for security auditing
10. **Implement rate limiting** to prevent brute force attacks