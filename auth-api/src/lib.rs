mod error;
pub mod services;
mod state;

pub use error::AuthError;
use services::keycloak::{ClientId, ClientSecret, Realm};
use state::AuthState;
use axum::Router;
use url::Url;

pub struct AuthApiConfig {
    pub keycloak_base_url: Url,
    pub keycloak_realm: Realm,
    pub keycloak_client_id: ClientId,
    pub keycloak_client_secret: ClientSecret,
    pub kafka_bootstrap_server: Url,
}

pub async fn create_app_router(config: AuthApiConfig) -> Result<Router, AuthError> {
    let state = AuthState::new(config)?;
    Ok(Router::new().with_state(state))
}
