mod config;

use crate::config::AppConfig;
use axum::serve;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use auth_api::{AuthApiConfig, create_app_router};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), u32> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("logger failed");

    dotenvy::dotenv().ok();

    let config = match AppConfig::new() {
        Ok(config) => config,
        Err(err) => {
            error!("Application configuration failed: {}", err);
            return Err(1);
        }
    };

    info!("Application configuration succeeded.");

    let auth_config = AuthApiConfig{
        keycloak_base_url: config.keycloak.base_url.clone(),
        keycloak_realm: config.keycloak.realm.clone(),
        keycloak_client_id: config.keycloak.client_id.clone(),
        keycloak_client_secret: config.keycloak.client_secret.clone(),
        kafka_bootstrap_server: config.kafka.bootstrap_server.clone(),
    };
    let auth_router = match create_app_router(auth_config).await {
        Ok(router) => router,
        Err(e) => {
            error!("Failed to create router: {}", e);
            return Err(1);
        }
    };

    let addr = format!("0.0.0.0:{}", config.server.port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind address {}: {}", addr, e);
            return Err(1);
        }
    };

    info!("HTTP-server started on http://{}", addr);

    if let Err(e) = serve(listener, auth_router).await {
        error!("Server failed because of: {}", e);
    }
    Ok(())
}