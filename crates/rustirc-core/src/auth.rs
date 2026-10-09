//! SASL Authentication Implementation
//!
//! This module provides SASL authentication support for IRC connections,
//! implementing PLAIN, EXTERNAL, and SCRAM-SHA-256 mechanisms as specified
//! in Phase 2 requirements.

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// SASL authentication state
#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    /// Not started
    Idle,
    /// Authentication in progress
    InProgress,
    /// Authentication completed successfully
    Success,
    /// Authentication failed
    Failed(String),
}

/// SASL credentials with secure password storage
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct SaslCredentials {
    #[allow(unused_assignments)] // Zeroize derive generates field assignments for security
    pub username: String,
    #[allow(unused_assignments)] // Zeroize derive generates field assignments for security
    pub password: SecureString,
    #[allow(unused_assignments)] // Zeroize derive generates field assignments for security
    pub authzid: Option<String>,
}

/// Secure string that automatically zeroes memory on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureString {
    #[allow(unused_assignments)] // Zeroize derive generates field assignments for security
    inner: Vec<u8>,
}

impl SecureString {
    pub fn new(s: String) -> Self {
        Self {
            inner: s.into_bytes(),
        }
    }

    pub fn as_str(&self) -> &str {
        // SAFETY: We only create SecureString from valid UTF-8 strings
        unsafe { std::str::from_utf8_unchecked(&self.inner) }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }
}

impl std::fmt::Debug for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecureString([REDACTED])")
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

/// SASL authentication mechanism trait
pub trait SaslMechanism: Send + Sync {
    fn name(&self) -> &str;
    fn initial_response(&self, credentials: &SaslCredentials) -> Result<Vec<u8>>;
    fn continue_auth(&mut self, challenge: Option<&[u8]>) -> Result<Vec<u8>>;
}

/// PLAIN authentication mechanism
pub struct PlainMechanism;

impl SaslMechanism for PlainMechanism {
    fn name(&self) -> &str {
        "PLAIN"
    }

    fn initial_response(&self, credentials: &SaslCredentials) -> Result<Vec<u8>> {
        let authzid = credentials.authzid.as_deref().unwrap_or("");
        let authcid = &credentials.username;
        let password = credentials.password.as_str();

        // Format: authzid\0authcid\0password
        let mut response = Vec::new();
        response.extend_from_slice(authzid.as_bytes());
        response.push(0);
        response.extend_from_slice(authcid.as_bytes());
        response.push(0);
        response.extend_from_slice(password.as_bytes());

        Ok(response)
    }

    fn continue_auth(&mut self, _challenge: Option<&[u8]>) -> Result<Vec<u8>> {
        // PLAIN doesn't use challenges
        Ok(Vec::new())
    }
}

/// EXTERNAL authentication mechanism
pub struct ExternalMechanism;

impl SaslMechanism for ExternalMechanism {
    fn name(&self) -> &str {
        "EXTERNAL"
    }

    fn initial_response(&self, credentials: &SaslCredentials) -> Result<Vec<u8>> {
        // Send authzid or empty string
        Ok(credentials
            .authzid
            .as_deref()
            .unwrap_or("")
            .as_bytes()
            .to_vec())
    }

    fn continue_auth(&mut self, _challenge: Option<&[u8]>) -> Result<Vec<u8>> {
        // EXTERNAL doesn't use challenges
        Ok(Vec::new())
    }
}

// SCRAM-SHA-256 helper functions
fn pbkdf2_sha256(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(password).expect("HMAC key");
    mac.update(salt);
    mac.update(&[0, 0, 0, 1]);
    let mut u = mac.finalize().into_bytes();
    let mut result = u;

    for _ in 1..iterations {
        let mut mac = HmacSha256::new_from_slice(password).expect("HMAC key");
        mac.update(&u);
        u = mac.finalize().into_bytes();
        for (r, byte) in result.iter_mut().zip(u.iter()) {
            *r ^= byte;
        }
    }

    result.into()
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

fn sha256_digest(data: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

fn generate_scram_nonce() -> String {
    use rand::RngExt;
    let mut rng = rand::rng();
    let random_bytes: [u8; 18] = rng.random();
    BASE64.encode(random_bytes)
}

fn parse_scram_attributes(input: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    for part in input.split(',') {
        if let Some((k, v)) = part.split_once('=') {
            attrs.insert(k.to_string(), v.to_string());
        }
    }
    attrs
}

#[derive(Debug, PartialEq)]
enum ScramState {
    Initial,
    ClientFirstSent,
    ClientFinalSent,
    Complete,
}

struct ScramInner {
    client_nonce: String,
    client_first_bare: String,
    password: Vec<u8>,
    server_signature: Option<[u8; 32]>,
    state: ScramState,
}

/// SCRAM-SHA-256 authentication mechanism (RFC 5802 / RFC 7677)
pub struct ScramSha256Mechanism {
    inner: std::sync::Mutex<ScramInner>,
}

impl Default for ScramSha256Mechanism {
    fn default() -> Self {
        Self::new()
    }
}

impl ScramSha256Mechanism {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(ScramInner {
                client_nonce: String::new(),
                client_first_bare: String::new(),
                password: Vec::new(),
                server_signature: None,
                state: ScramState::Initial,
            }),
        }
    }

    /// Testing helper to set fixed client nonce
    pub fn with_fixed_nonce(nonce: String) -> Self {
        Self {
            inner: std::sync::Mutex::new(ScramInner {
                client_nonce: nonce,
                client_first_bare: String::new(),
                password: Vec::new(),
                server_signature: None,
                state: ScramState::Initial,
            }),
        }
    }
}

impl SaslMechanism for ScramSha256Mechanism {
    fn name(&self) -> &str {
        "SCRAM-SHA-256"
    }

    fn initial_response(&self, credentials: &SaslCredentials) -> Result<Vec<u8>> {
        let mut inner = self.inner.lock().unwrap();
        let client_nonce = if inner.client_nonce.is_empty() {
            generate_scram_nonce()
        } else {
            inner.client_nonce.clone()
        };

        let username = credentials.username.replace('=', "=3D").replace(',', "=2C");
        let client_first_bare = format!("n={},r={}", username, client_nonce);
        let client_first = format!("n,,{}", client_first_bare);

        inner.client_nonce = client_nonce;
        inner.client_first_bare = client_first_bare;
        inner.password = credentials.password.as_bytes().to_vec();
        inner.state = ScramState::ClientFirstSent;

        Ok(client_first.into_bytes())
    }

    fn continue_auth(&mut self, challenge: Option<&[u8]>) -> Result<Vec<u8>> {
        let mut inner = self.inner.lock().unwrap();
        let challenge_bytes =
            challenge.ok_or_else(|| anyhow::anyhow!("Missing challenge in SCRAM-SHA-256"))?;
        let challenge_str = std::str::from_utf8(challenge_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in SCRAM challenge: {e}"))?;

        match inner.state {
            ScramState::ClientFirstSent => {
                let attrs = parse_scram_attributes(challenge_str);
                let server_nonce = attrs
                    .get("r")
                    .ok_or_else(|| anyhow::anyhow!("Missing nonce in server-first message"))?;
                if !server_nonce.starts_with(&inner.client_nonce) {
                    return Err(anyhow::anyhow!("Server nonce does not extend client nonce"));
                }

                let salt_str = attrs
                    .get("s")
                    .ok_or_else(|| anyhow::anyhow!("Missing salt in server-first message"))?;
                let salt = BASE64
                    .decode(salt_str)
                    .map_err(|e| anyhow::anyhow!("Invalid base64 salt: {e}"))?;

                let iterations = attrs
                    .get("i")
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing iteration count in server-first message")
                    })?
                    .parse::<u32>()
                    .map_err(|e| anyhow::anyhow!("Invalid iteration count: {e}"))?;

                if iterations == 0 {
                    return Err(anyhow::anyhow!("Iteration count must be > 0"));
                }

                let channel_binding = "c=biws";
                let client_final_without_proof = format!("{},r={}", channel_binding, server_nonce);

                let salted_password = pbkdf2_sha256(&inner.password, &salt, iterations);
                let client_key = hmac_sha256(&salted_password, b"Client Key");
                let stored_key = sha256_digest(&client_key);

                let auth_message = format!(
                    "{},{},{}",
                    inner.client_first_bare, challenge_str, client_final_without_proof
                );

                let client_signature = hmac_sha256(&stored_key, auth_message.as_bytes());
                let mut client_proof = [0u8; 32];
                for i in 0..32 {
                    client_proof[i] = client_key[i] ^ client_signature[i];
                }

                let server_key = hmac_sha256(&salted_password, b"Server Key");
                let server_signature = hmac_sha256(&server_key, auth_message.as_bytes());

                let client_final = format!(
                    "{},p={}",
                    client_final_without_proof,
                    BASE64.encode(client_proof)
                );

                inner.server_signature = Some(server_signature);
                inner.state = ScramState::ClientFinalSent;

                Ok(client_final.into_bytes())
            }
            ScramState::ClientFinalSent => {
                let attrs = parse_scram_attributes(challenge_str);
                let v = attrs.get("v").ok_or_else(|| {
                    anyhow::anyhow!("Missing server signature in server-final message")
                })?;
                let received_sig = BASE64
                    .decode(v)
                    .map_err(|e| anyhow::anyhow!("Invalid base64 server signature: {e}"))?;

                let expected_sig = inner
                    .server_signature
                    .ok_or_else(|| anyhow::anyhow!("Missing expected server signature"))?;

                if received_sig != expected_sig {
                    return Err(anyhow::anyhow!("Server signature verification failed"));
                }

                inner.state = ScramState::Complete;
                Ok(Vec::new())
            }
            ScramState::Complete => Ok(Vec::new()),
            ScramState::Initial => Err(anyhow::anyhow!("SCRAM authentication not initiated")),
        }
    }
}

/// SASL authenticator managing the authentication flow
pub struct SaslAuthenticator {
    state: AuthState,
    mechanisms: HashMap<String, Box<dyn SaslMechanism>>,
    current_mechanism: Option<String>,
}

impl Default for SaslAuthenticator {
    fn default() -> Self {
        Self::new()
    }
}

impl SaslAuthenticator {
    pub fn new() -> Self {
        let mut auth = Self {
            state: AuthState::Idle,
            mechanisms: HashMap::new(),
            current_mechanism: None,
        };

        // Register built-in mechanisms
        auth.register_mechanism("PLAIN", Box::new(PlainMechanism));
        auth.register_mechanism("EXTERNAL", Box::new(ExternalMechanism));
        auth.register_mechanism("SCRAM-SHA-256", Box::new(ScramSha256Mechanism::new()));

        auth
    }

    pub fn register_mechanism(&mut self, name: &str, mechanism: Box<dyn SaslMechanism>) {
        self.mechanisms.insert(name.to_string(), mechanism);
    }

    pub fn get_available_mechanisms(&self) -> Vec<String> {
        self.mechanisms.keys().cloned().collect()
    }

    pub fn state(&self) -> &AuthState {
        &self.state
    }

    pub async fn start_authentication(
        &mut self,
        mechanism: &str,
        credentials: SaslCredentials,
    ) -> Result<Vec<u8>> {
        debug!("Starting SASL authentication with mechanism: {mechanism}");

        if !self.mechanisms.contains_key(mechanism) {
            let error = format!("Unsupported SASL mechanism: {mechanism}");
            self.state = AuthState::Failed(error.clone());
            return Err(anyhow::anyhow!(error));
        }

        self.current_mechanism = Some(mechanism.to_string());
        self.state = AuthState::InProgress;

        let mechanism_impl = self
            .mechanisms
            .get(mechanism)
            .ok_or_else(|| anyhow::anyhow!("Mechanism disappeared during authentication"))?;
        let response = mechanism_impl.initial_response(&credentials)?;

        debug!("Generated initial response of {} bytes", response.len());
        Ok(response)
    }

    pub async fn continue_authentication(&mut self, challenge: Option<&[u8]>) -> Result<Vec<u8>> {
        if let Some(mechanism_name) = &self.current_mechanism {
            if let Some(mechanism) = self.mechanisms.get_mut(mechanism_name) {
                let response = mechanism.continue_auth(challenge)?;
                debug!("Generated challenge response of {} bytes", response.len());
                return Ok(response);
            }
        }

        Err(anyhow::anyhow!("No authentication in progress"))
    }

    pub fn handle_success(&mut self) {
        info!("SASL authentication successful");
        self.state = AuthState::Success;
        self.current_mechanism = None;
    }

    pub fn handle_failure(&mut self, reason: Option<&str>) {
        let error = reason.unwrap_or("Authentication failed").to_string();
        warn!("SASL authentication failed: {error}");
        self.state = AuthState::Failed(error);
        self.current_mechanism = None;
    }

    pub fn reset(&mut self) {
        self.state = AuthState::Idle;
        self.current_mechanism = None;
    }
}

/// Encode data for AUTHENTICATE command
pub fn encode_authenticate_data(data: &[u8]) -> String {
    if data.is_empty() {
        "+".to_string()
    } else {
        BASE64.encode(data)
    }
}

/// Decode data from AUTHENTICATE command
pub fn decode_authenticate_data(data: &str) -> Result<Vec<u8>> {
    if data == "+" {
        Ok(Vec::new())
    } else {
        BASE64
            .decode(data.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to decode base64: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_mechanism() {
        let plain = PlainMechanism;
        let creds = SaslCredentials {
            username: "user".to_string(),
            password: "pass".into(),
            authzid: None,
        };

        let response = plain.initial_response(&creds).unwrap();
        let expected = b"\0user\0pass";
        assert_eq!(response, expected);
    }

    #[test]
    fn test_plain_mechanism_with_authzid() {
        let plain = PlainMechanism;
        let creds = SaslCredentials {
            username: "user".to_string(),
            password: "pass".into(),
            authzid: Some("admin".to_string()),
        };

        let response = plain.initial_response(&creds).unwrap();
        let expected = b"admin\0user\0pass";
        assert_eq!(response, expected);
    }

    #[test]
    fn test_external_mechanism() {
        let external = ExternalMechanism;
        let creds = SaslCredentials {
            username: "user".to_string(),
            password: "pass".into(),
            authzid: Some("admin".to_string()),
        };

        let response = external.initial_response(&creds).unwrap();
        assert_eq!(response, b"admin");
    }

    #[test]
    fn test_encode_decode_authenticate() {
        let data = b"test data";
        let encoded = encode_authenticate_data(data);
        let decoded = decode_authenticate_data(&encoded).unwrap();
        assert_eq!(decoded, data);

        // Test empty data
        let empty_encoded = encode_authenticate_data(&[]);
        assert_eq!(empty_encoded, "+");
        let empty_decoded = decode_authenticate_data("+").unwrap();
        assert_eq!(empty_decoded, Vec::<u8>::new());
    }

    #[tokio::test]
    async fn test_authenticator_flow() {
        let mut auth = SaslAuthenticator::new();

        let creds = SaslCredentials {
            username: "testuser".to_string(),
            password: "testpass".into(),
            authzid: None,
        };

        // Start authentication
        let response = auth.start_authentication("PLAIN", creds).await.unwrap();
        assert_eq!(auth.state(), &AuthState::InProgress);
        assert_eq!(response, b"\0testuser\0testpass");

        // Simulate success
        auth.handle_success();
        assert_eq!(auth.state(), &AuthState::Success);
    }

    #[test]
    fn test_secure_string_zeroization() {
        use std::mem::ManuallyDrop;
        use zeroize::Zeroize;

        // Create a SecureString with sensitive data
        let sensitive_data = "password123".to_string();
        let mut secure_str = ManuallyDrop::new(SecureString::new(sensitive_data.clone()));

        // Verify the data is present before zeroization
        assert_eq!(secure_str.as_str(), "password123");

        // Manually call zeroize (this is what ZeroizeOnDrop does in Drop)
        secure_str.zeroize();

        // After zeroization, the inner bytes should be zeroed
        assert!(
            secure_str.inner.iter().all(|&b| b == 0),
            "SecureString memory should be zeroed after calling zeroize()"
        );

        // Clean up the ManuallyDrop wrapper
        unsafe {
            ManuallyDrop::drop(&mut secure_str);
        }
    }

    #[test]
    fn test_sasl_credentials_zeroization() {
        use std::mem::ManuallyDrop;
        use zeroize::Zeroize;

        // Create SaslCredentials with sensitive data
        let username = "testuser".to_string();
        let password = SecureString::new("secretpass".to_string());
        let authzid = Some("admin".to_string());

        let mut creds = ManuallyDrop::new(SaslCredentials {
            username: username.clone(),
            password: password.clone(),
            authzid: authzid.clone(),
        });

        // Verify the data is present before zeroization
        assert_eq!(creds.username, "testuser");
        assert_eq!(creds.password.as_str(), "secretpass");
        assert_eq!(creds.authzid.as_deref(), Some("admin"));

        // Manually call zeroize (this is what ZeroizeOnDrop does in Drop)
        creds.zeroize();

        // After zeroization, all string fields should be zeroed
        assert!(
            creds.username.as_bytes().iter().all(|&b| b == 0),
            "SaslCredentials username should be zeroed after calling zeroize()"
        );
        assert!(
            creds.password.inner.iter().all(|&b| b == 0),
            "SaslCredentials password should be zeroed after calling zeroize()"
        );
        if let Some(ref authzid_val) = creds.authzid {
            assert!(
                authzid_val.as_bytes().iter().all(|&b| b == 0),
                "SaslCredentials authzid should be zeroed after calling zeroize()"
            );
        }

        // Clean up the ManuallyDrop wrapper
        unsafe {
            ManuallyDrop::drop(&mut creds);
        }
    }

    #[test]
    fn test_secure_string_zeroize_on_drop() {
        // This test verifies that ZeroizeOnDrop is properly derived
        // by ensuring the struct implements the trait
        fn assert_zeroize_on_drop<T: zeroize::ZeroizeOnDrop>() {}
        assert_zeroize_on_drop::<SecureString>();
    }

    #[test]
    fn test_sasl_credentials_zeroize_on_drop() {
        // This test verifies that ZeroizeOnDrop is properly derived
        // by ensuring the struct implements the trait
        fn assert_zeroize_on_drop<T: zeroize::ZeroizeOnDrop>() {}
        assert_zeroize_on_drop::<SaslCredentials>();
    }

    #[test]
    fn test_scram_sha256_rfc7677() {
        let mut mech = ScramSha256Mechanism::with_fixed_nonce("rOprNGfwEbeRWgbNEkqO".to_string());
        let creds = SaslCredentials {
            username: "user".to_string(),
            password: SecureString::new("pencil".to_string()),
            authzid: None,
        };

        // 1. Initial client-first message
        let client_first = mech.initial_response(&creds).unwrap();
        assert_eq!(
            std::str::from_utf8(&client_first).unwrap(),
            "n,,n=user,r=rOprNGfwEbeRWgbNEkqO"
        );

        // 2. Server-first challenge (RFC 7677 Section 3)
        let server_first =
            b"r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096";
        let client_final = mech.continue_auth(Some(server_first)).unwrap();
        assert_eq!(
            std::str::from_utf8(&client_final).unwrap(),
            "c=biws,r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,p=dHzbZapWIk4jUhN+Ute9ytag9zjfMHgsqmmiz7AndVQ="
        );

        // 3. Server-final challenge (RFC 7677 Section 3)
        let server_final = b"v=6rriTRBi23WpRR/wtup+mMhUZUn/dB5nLTJRsjl95G4=";
        let final_resp = mech.continue_auth(Some(server_final)).unwrap();
        assert!(final_resp.is_empty());
    }

    #[tokio::test]
    async fn test_scram_sha256_authenticator_registration() {
        let auth = SaslAuthenticator::new();
        let mechs = auth.get_available_mechanisms();
        assert!(mechs.contains(&"SCRAM-SHA-256".to_string()));
        assert!(mechs.contains(&"PLAIN".to_string()));
        assert!(mechs.contains(&"EXTERNAL".to_string()));
    }
}
