use secrecy::{ExposeSecret, SecretString};
use std::fmt::{Debug, Formatter};

#[derive(Debug, thiserror::Error)]
pub enum KeycloakError {
    #[error("{0} cannot be empty")]
    Empty(&'static str),
    #[error("Invalid character '{0}' in realm name")]
    InvalidRealmChar(char),
    #[error("Client secret is too short to be valid (minimum {min} characters)")]
    SecretTooShort { min: usize },
}

#[derive(Clone, Debug)]
pub struct Realm(String);

impl AsRef<str> for Realm {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Realm {
    pub fn new(s: &str) -> Result<Self, KeycloakError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(KeycloakError::Empty("realm"));
        }
        for c in trimmed.chars() {
            if !c.is_ascii_alphanumeric() && c != '-' && c != '_' {
                return Err(KeycloakError::InvalidRealmChar(c));
            }
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Clone, Debug)]
pub struct ClientId(String);

impl AsRef<str> for ClientId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ClientId {
    pub fn new(s: &str) -> Result<Self, KeycloakError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(KeycloakError::Empty("client_id"));
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Clone)]
pub struct ClientSecret(SecretString);

impl Debug for ClientSecret {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("*client_secret*")
    }
}

impl ClientSecret {
    /// Minimal length below which a client secret is almost certainly a
    /// misconfiguration, not a real Keycloak-issued secret.
    /// Not tied to any particular secret format.
    const MIN_SECRET_LENGTH: usize = 16;

    pub fn new(s: &str) -> Result<Self, KeycloakError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(KeycloakError::Empty("client_secret"));
        }
        if trimmed.len() < Self::MIN_SECRET_LENGTH {
            return Err(KeycloakError::SecretTooShort {
                min: Self::MIN_SECRET_LENGTH,
            });
        }
        Ok(Self(SecretString::from(trimmed.to_string())))
    }

    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

#[derive(Clone)]
pub(crate) struct KeycloakState {
    base_url: url::Url,
    realm: Realm,
    client_id: ClientId,
    client_secret: ClientSecret,
}

impl KeycloakState {
    pub fn new(
        base_url: url::Url,
        realm: Realm,
        client_id: ClientId,
        client_secret: ClientSecret,
    ) -> Self {
        Self {
            base_url,
            realm,
            client_id,
            client_secret,
        }
    }
}