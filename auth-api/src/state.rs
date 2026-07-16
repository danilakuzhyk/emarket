use crate::AuthApiConfig;
use crate::error::AuthInitError;
use crate::services::kafka::KafkaState;
use crate::services::keycloak::KeycloakState;

#[derive(Clone)]
pub(crate) struct AuthState {
    keycloak: KeycloakState,
    kafka: KafkaState,
}

impl AuthState {
    pub fn new(config: AuthApiConfig) -> Result<Self, AuthInitError> {
        Ok(AuthState {
            keycloak: KeycloakState::new(
                config.keycloak_base_url,
                config.keycloak_realm,
                config.keycloak_client_id,
                config.keycloak_client_secret,
            ),
            kafka: KafkaState::new(&config.kafka_bootstrap_server)?,
        })
    }
}
