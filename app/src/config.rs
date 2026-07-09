use serde::Deserialize;
use config::{Config, ConfigError, Environment};

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeycloakConfig {
    pub base_url: String,
    pub realm: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaConfig {
    pub bootstrap_server: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub keycloak: KeycloakConfig,
    pub kafka: KafkaConfig,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .set_default("server.port", 3000)?

            .set_default("keycloak.base_url", "http://localhost:8080")?
            .set_default("keycloak.realm", "emarket")?
            .set_default("keycloak.client_id", "emarket-app")?
            .set_default("keycloak.client_secret", "secret")?
            .set_default("kafka.boostrap_server", "localhost:9092")?

            .add_source(
                Environment::with_prefix("APP")
            )
            .build()?;
        settings.try_deserialize::<Self>()
    }
}