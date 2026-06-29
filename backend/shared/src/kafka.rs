use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

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

impl Default for SharedKafkaProducer {
    fn default() -> Self {
        Self::new("localhost:9092")
    }
}

impl SharedKafkaProducer {
    pub fn new(bootstrap_servers: &str) -> Self {
        let producer: FutureProducer = base_config(bootstrap_servers)
            .create()
            .expect("failed to create a producer");
        Self { producer }
    }

    pub async fn send_internal<T>(&self, topic: &str, key: &str, payload: &T) -> Result<(), String>
    where
        T: serde::Serialize,
    {
        let payload_bytes = match serde_json::to_vec(payload) {
            Ok(bytes) => bytes,
            Err(error) => return Err(error.to_string()),
        };

        match self
            .producer
            .send(
                FutureRecord::to(topic).payload(&payload_bytes).key(key),
                Duration::from_secs(5),
            )
            .await
        {
            Ok(_) => Ok(()),
            Err((kafka_error, _)) => Err(format!("Error Kafka: {}", kafka_error)),
        }
    }
}
