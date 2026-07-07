use crate::error::AppError;
use reqwest::Client;

#[derive(Clone)]
pub struct KeycloakState {
    http_client: Client,
    keycloak_base_url: String,
    keycloak_realm: String,
    keycloak_client_id: String,
    keycloak_client_secret: String,
}

impl KeycloakState {
    pub async fn from_env() -> Result<Self, AppError> {
        let keycloak_base_url = std::env::var("KEYCLOAK_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let keycloak_realm =
            std::env::var("KEYCLOAK_REALM").unwrap_or_else(|_| "emarket".to_string());
        let keycloak_client_id =
            std::env::var("KEYCLOAK_CLIENT_ID").unwrap_or_else(|_| "emarket-app".to_string());
        let keycloak_client_secret = std::env::var("KEYCLOAK_CLIENT_SECRET")
            .unwrap_or_else(|_| "nqLfnsT8VqSoB4Pl3Wq6c7QtznfayvDf".to_string());

        let http_client = Client::new();

        Ok(Self {
            http_client,
            keycloak_base_url,
            keycloak_realm,
            keycloak_client_id,
            keycloak_client_secret,
        })

        Self { base_url }
    }
}