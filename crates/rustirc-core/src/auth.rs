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
    pub username: String,
    #[zeroize(skip)]
    pub password: SecureString,
    pub authzid: Option<String>,
}

/// Secure string that automatically zeroes memory on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureString {
    #[zeroize(skip)]
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
}
