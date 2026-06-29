use crate::services::kafka::KafkaState;
use crate::services::keycloak::KeycloakState;

#[derive(Default, Clone)]
pub struct AppState {
    pub(crate) kafka_state: KafkaState,
    pub(crate) keycloak_state: KeycloakState,
}
