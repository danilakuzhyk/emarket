mod services;
mod state;

use axum::Router;
use crate::state::AuthState;

pub struct AuthApiConfig {
    pub keycloak_base_url: String,
    pub keycloak_realm: String,
    pub keycloak_client_id: String,
    pub keycloak_client_secret: String,
    pub kafka_bootstrap_server: String,
}

pub async fn create_app_router(
    config: AuthApiConfig
) -> Result<Router, String> {
    let state = AuthState::new(config);
    Ok(Router::new().with_state(state))
}
