use app_config::AppConfig;
use auth_api::{AuthApiConfig, AuthInitError, create_app_router};
use axum::{Router, serve};
use std::process::ExitCode;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Debug, thiserror::Error)]
enum BootstrapError {
    #[error("Failed to run server: {0}")]
    Config(#[from] app_config::AppConfigError),

    #[error("Failed to create router: {0}")]
    Router(#[from] AuthInitError),

    #[error("Failed to bind address {addr}: {source}")]
    Bind {
        addr: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Server error: {0}")]
    Serve(#[source] std::io::Error),
}

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();
    init_tracing();

    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{}", err);
            ExitCode::FAILURE
        }
    }
}

fn init_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("logger initialization failed");
}

async fn run() -> Result<(), BootstrapError> {
    let config = AppConfig::new()?;
    info!("Application configuration succeeded.");

    let port = config.server.port;
    let router = build_router(config).await?;
    let listener = bind(port).await?;

    info!(
        "HTTP server started on http://{}",
        listener.local_addr().unwrap()
    );

    serve(listener, router)
        .await
        .map_err(BootstrapError::Serve)?;

    info!("Server shut down gracefully");
    Ok(())
}

async fn build_router(config: AppConfig) -> Result<Router, BootstrapError> {
    Ok(create_app_router(into_auth_config(config)).await?)
}

async fn bind(port: u16) -> Result<TcpListener, BootstrapError> {
    let addr = format!("0.0.0.0:{port}");
    TcpListener::bind(&addr)
        .await
        .map_err(|source| BootstrapError::Bind { addr, source })
}

fn into_auth_config(config: AppConfig) -> AuthApiConfig {
    AuthApiConfig {
        keycloak_base_url: config.keycloak.base_url,
        keycloak_realm: config.keycloak.realm,
        keycloak_client_id: config.keycloak.client_id,
        keycloak_client_secret: config.keycloak.client_secret,
        kafka_bootstrap_server: config.kafka.bootstrap_server,
    }
}