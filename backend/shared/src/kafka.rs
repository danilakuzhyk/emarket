use crate::kafka_events::{CustomerRegisteredEvent, VendorRegisteredEvent};
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

    pub async fn send_customer_registered(
        &self,
        event: &CustomerRegisteredEvent,
    ) -> Result<(), String> {
        let payload = match serde_json::to_string(event) {
            Ok(json_string) => json_string,
            Err(err) => return Err(format!("Serialization fault: {}", err)),
        };

        let record = FutureRecord::to("customer-registered")
            .payload(&payload)
            .key(&event.user_id);

        match self.producer.send(record, Duration::from_secs(0)).await {
            Ok(_) => Ok(()),
            Err((kafka_error, _)) => Err(format!("Error Kafka: {}", kafka_error)),
        }
    }

    pub async fn send_vendor_registered(
        &self,
        event: &VendorRegisteredEvent,
    ) -> Result<(), String> {
        let payload = match serde_json::to_string(event) {
            Ok(json_string) => json_string,
            Err(err) => return Err(format!("Serialization fault: {}", err)),
        };

        let record = FutureRecord::to("vendor-registered")
            .payload(&payload)
            .key(&event.user_id);

        match self.producer.send(record, Duration::from_secs(0)).await {
            Ok(_) => Ok(()),
            Err((kafka_error, _)) => Err(format!("Error Kafka: {}", kafka_error)),
        }
    }
}
