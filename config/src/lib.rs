use auth_api::services::{ClientId, ClientSecret, KeycloakError, Realm};
use config::{Config, Environment};
use serde::Deserialize;
use thiserror::Error;
use url::Url;

const DEFAULTS: &str = r#"
[server]
port = 3000

[keycloak]
base_url = "http://localhost:8080"
realm = "emarket"
client_id = "emarket-app"

[kafka]
bootstrap_server = "localhost:9092"
"#;

#[derive(Debug, Error)]
pub enum AppConfigError {
    #[error("Failed to load configuration: {0}")]
    Load(#[from] config::ConfigError),

    #[error("Invalid Keycloak base URL: {0}")]
    InvalidKeycloakUrl(#[source] url::ParseError),

    #[error("Invalid Kafka bootstrap server URL: {0}")]
    InvalidKafkaUrl(#[source] url::ParseError),

    #[error("Invalid Keycloak settings: {0}")]
    Keycloak(#[from] KeycloakError),
}

#[derive(Debug, Deserialize)]
struct RawAppConfig {
    server: RawServerConfig,
    keycloak: RawKeycloakConfig,
    kafka: RawKafkaConfig,
}

#[derive(Debug, Deserialize)]
struct RawServerConfig {
    port: u16,
}

#[derive(Debug, Deserialize)]
struct RawKeycloakConfig {
    base_url: String,
    realm: String,
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Deserialize)]
struct RawKafkaConfig {
    bootstrap_server: String,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct KeycloakConfig {
    pub base_url: Url,
    pub realm: Realm,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
}

#[derive(Debug, Clone)]
pub struct KafkaConfig {
    pub bootstrap_server: Url,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub keycloak: KeycloakConfig,
    pub kafka: KafkaConfig,
}

impl AppConfig {
    pub fn new() -> Result<Self, AppConfigError> {
        let raw = Self::load_raw()?;
        Self::try_from_raw(raw)
    }
    fn load_raw() -> Result<RawAppConfig, AppConfigError> {
        let settings = Config::builder()
            .add_source(config::File::from_str(DEFAULTS, config::FileFormat::Toml))
            .add_source(
                Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;

        Ok(settings.try_deserialize::<RawAppConfig>()?)
    }

    fn try_from_raw(raw: RawAppConfig) -> Result<Self, AppConfigError> {
        let keycloak = KeycloakConfig {
            base_url: Url::parse(&raw.keycloak.base_url)
                .map_err(AppConfigError::InvalidKeycloakUrl)?,
            realm: Realm::new(&raw.keycloak.realm)?,
            client_id: ClientId::new(&raw.keycloak.client_id)?,
            client_secret: ClientSecret::new(&raw.keycloak.client_secret)?,
        };

        let kafka = KafkaConfig {
            bootstrap_server: Url::parse(&raw.kafka.bootstrap_server)
                .map_err(AppConfigError::InvalidKafkaUrl)?,
        };

        Ok(Self {
            server: ServerConfig {
                port: raw.server.port,
            },
            keycloak,
            kafka,
        })
    }
}
