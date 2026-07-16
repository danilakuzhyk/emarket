use config_types::kafka::KafkaBootstrapServer;
use rdkafka::{
    config::ClientConfig,
    producer::{FutureProducer, FutureRecord},
};
use std::time::Duration;
use tracing::{debug, error, info};

#[derive(Debug, thiserror::Error)]
pub enum KafkaClientError {
    #[error("Failed to create Kafka producer: {0}")]
    Initialization(#[from] rdkafka::error::KafkaError),

    #[error("Failed to serialize Kafka payload: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Kafka send error on topic {topic}: {source}")]
    Publish {
        topic: String,
        source: rdkafka::error::KafkaError,
    },
}

fn base_config(bootstrap_server: &KafkaBootstrapServer) -> ClientConfig {
    let mut config = ClientConfig::new();
    config
        .set("bootstrap.servers", bootstrap_server.as_host_port())
        .set("message.timeout.ms", "5000");
    config
}

#[derive(Clone)]
pub struct SharedKafkaProducer {
    producer: FutureProducer,
}

impl SharedKafkaProducer {
    pub fn new(bootstrap_server: &KafkaBootstrapServer) -> Result<Self, KafkaClientError> {
        let producer: FutureProducer = base_config(bootstrap_server)
            .create()
            .inspect_err(|e| error!("Failed to create Kafka producer: {}", e))?;

        info!(
            "Kafka producer successfully initialized for: {}",
            bootstrap_server
        );
        Ok(Self { producer })
    }

    pub async fn send_internal<T>(
        &self,
        topic: &str,
        key: &str,
        payload: &T,
    ) -> Result<(), KafkaClientError>
    where
        T: serde::Serialize,
    {
        let payload_bytes = serde_json::to_vec(payload)
            .inspect_err(|err| error!("Failed to serialize Kafka payload: {}", err))?;

        self.producer
            .send(
                FutureRecord::to(topic).payload(&payload_bytes).key(key),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(kafka_error, _)| KafkaClientError::Publish {
                topic: topic.to_string(),
                source: kafka_error,
            })
            .inspect_err(|err| error!("{}", err))?;

        debug!("Message successfully sent to topic: {}", topic);
        Ok(())
    }
}
