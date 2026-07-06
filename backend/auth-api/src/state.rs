use crate::services::kafka::KafkaState;
use crate::services::keycloak::KeycloakState;
use crate::error::AppError;

#[derive(Clone)]
pub struct AppState {
    pub(crate) kafka_state: KafkaState,
    pub(crate) keycloak_state: KeycloakState,
}

impl AppState {
    pub async fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            kafka_state: KafkaState::from_env(),
            keycloak_state: KeycloakState::from_env().await?,
        })
    }
}
