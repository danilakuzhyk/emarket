use rdkafka::{
    config::ClientConfig,
    producer::{FutureProducer, FutureRecord},
};
use std::time::Duration;
use tracing::{info, error};

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
    pub fn new(bootstrap_servers: &str) -> Result<Self, String> {
        let producer: FutureProducer = base_config(bootstrap_servers)
            .create()
            .map_err(|e| format!("Failed to create Kafka producer: {}", e))?;

        info!("Kafka producer successfully initialized for: {}", bootstrap_servers);
        Ok(Self { producer })
    }

    pub async fn send_internal<T>(&self, topic: &str, key: &str, payload: &T) -> Result<(), String>
    where
        T: serde::Serialize,
    {
        let payload_bytes = match serde_json::to_vec(payload) {
            Ok(bytes) => bytes,
            Err(error) => {
                error!("Failed to serialize Kafka payload: {}", error);
                return Err(error.to_string());
            }
        };

        match self
            .producer
            .send(
                FutureRecord::to(topic).payload(&payload_bytes).key(key),
                Duration::from_secs(5),
            )
            .await
        {
            Ok(_) => {
                tracing::debug!("Message successfully sent to topic: {}", topic);
                Ok(())
            }
            Err((kafka_error, _)) => {
                error!("Kafka send error on topic {}: {}", topic, kafka_error);
                Err(format!("Error Kafka: {}", kafka_error))
            }
        }
    }
}