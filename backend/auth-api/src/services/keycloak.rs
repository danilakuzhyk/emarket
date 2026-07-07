#[derive(Clone)]
pub struct KeycloakClient {
    pub base_url: String,
}

impl KeycloakClient {
    pub fn from_env() -> Self {
        let base_url = std::env::var("KEYCLOAK_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        Self { base_url }
    }
}