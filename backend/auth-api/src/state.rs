use crate::error::AppError;
use crate::services::kafka::KafkaState;
use crate::services::keycloak::KeycloakState;

#[derive(Clone)]
pub struct AppState {
    pub(crate) keycloak_state: KeycloakState,
    pub(crate) kafka_state: KafkaState,
}

impl AppState {
    pub async fn from_env() -> Result<AppState, AppError> {
        Ok(Self {
            keycloak_state: KeycloakState::from_env().await?,
            kafka_state: KafkaState::from_env()?,
        })
    }
}