mod error;
pub mod services;
mod state;

pub use error::{AuthError, AuthInitError};

use axum::Router;
use config_types::kafka::KafkaBootstrapServer;
use config_types::keycloak::{BaseUrl, ClientId, ClientSecret, Realm};
use state::AuthState;

pub struct AuthApiConfig {
    pub keycloak_base_url: BaseUrl,
    pub keycloak_realm: Realm,
    pub keycloak_client_id: ClientId,
    pub keycloak_client_secret: ClientSecret,
    pub kafka_bootstrap_server: KafkaBootstrapServer,
}

pub async fn create_app_router(config: AuthApiConfig) -> Result<Router, AuthInitError> {
    let state = AuthState::new(config)?;
    Ok(Router::new().with_state(state))
}
