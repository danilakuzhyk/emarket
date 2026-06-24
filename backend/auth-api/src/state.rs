use crate::services::keycloak::KeycloakState;

#[derive(Default, Clone)]
pub struct AppState {
    pub(crate) keycloak_state: KeycloakState,
}
