use crate::error::AppError;
use crate::services::keycloak::KeycloakState;

#[derive(Clone)]
pub struct AppState {
    pub(crate) keycloak_state: KeycloakState,
}

impl AppState {
    pub async fn from_env() -> Result<AppState, AppError> {
        Ok(Self {
            keycloak_state: KeycloakState::from_env().await?,
        })
    }
}