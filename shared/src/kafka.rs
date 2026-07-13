use rdkafka::{
    config::ClientConfig,
    producer::{FutureProducer, FutureRecord},
};
use std::time::Duration;
use tracing::{info, error, debug};

#[derive(Debug, thiserror::Error)]
pub enum KafkaError {
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

fn base_config(bootstrap_servers: &str) -> ClientConfig {
    let mut config = ClientConfig::new();
    config
        .set("bootstrap.servers", bootstrap_servers)
        .set("message.timeout.ms", "5000");
    config
}

#[derive(Clone)]
pub struct SharedKafkaProducer {
    producer: FutureProducer,
}

impl SharedKafkaProducer {
    pub fn new(bootstrap_servers: &str) -> Result<Self, KafkaError> {
        let producer: FutureProducer = base_config(bootstrap_servers)
            .create()
            .inspect_err(|e| error!("Failed to create Kafka producer: {}", e))?;

        info!("Kafka producer successfully initialized for: {}", bootstrap_servers);
        Ok(Self { producer })
    }

    pub async fn send_internal<T>(&self, topic: &str, key: &str, payload: &T) -> Result<(), KafkaError>
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
            .map_err(|(kafka_error, _)| KafkaError::Publish {
                topic: topic.to_string(),
                source: kafka_error,
            })
            .inspect_err(|err| error!("{}", err))?;

        debug!("Message successfully sent to topic: {}", topic);
        Ok(())
    }
}