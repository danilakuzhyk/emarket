use secrecy::{ExposeSecret, SecretString};
use std::fmt::{Debug, Formatter};
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum KeycloakConfigError {
    #[error("{0} cannot be empty")]
    Empty(&'static str),
    #[error("Invalid character '{0}' in realm name")]
    InvalidRealmChar(char),
    #[error("Client secret is too short to be valid (minimum {min} characters)")]
    SecretTooShort { min: usize },
    #[error("Keycloak base URL '{0}' must use http or https scheme")]
    InvalidBaseUrlScheme(Url),
    #[error("Keycloak base URL '{0}' is missing a host")]
    MissingBaseUrlHost(Url),
}

#[derive(Clone, Debug)]
pub struct BaseUrl(Url);

impl BaseUrl {
    pub fn new(url: Url) -> Result<Self, KeycloakConfigError> {
        if url.scheme() != "http" && url.scheme() != "https" {
            return Err(KeycloakConfigError::InvalidBaseUrlScheme(url));
        }
        if url.host_str().is_none() {
            return Err(KeycloakConfigError::MissingBaseUrlHost(url));
        }
        Ok(Self(url))
    }

    pub fn as_url(&self) -> &Url {
        &self.0
    }
}

impl std::fmt::Display for BaseUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct Realm(String);

impl AsRef<str> for Realm {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Realm {
    pub fn new(s: &str) -> Result<Self, KeycloakConfigError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(KeycloakConfigError::Empty("realm"));
        }
        for c in trimmed.chars() {
            if !c.is_ascii_alphanumeric() && c != '-' && c != '_' {
                return Err(KeycloakConfigError::InvalidRealmChar(c));
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
    pub fn new(s: &str) -> Result<Self, KeycloakConfigError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(KeycloakConfigError::Empty("client_id"));
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

    pub fn new(s: &str) -> Result<Self, KeycloakConfigError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(KeycloakConfigError::Empty("client_secret"));
        }
        if trimmed.len() < Self::MIN_SECRET_LENGTH {
            return Err(KeycloakConfigError::SecretTooShort {
                min: Self::MIN_SECRET_LENGTH,
            });
        }
        Ok(Self(SecretString::from(trimmed.to_string())))
    }

    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}
