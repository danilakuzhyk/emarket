#[derive(Clone)]
pub struct KafkaProducer {
    pub brokers: String,
}

impl KafkaProducer {
    pub fn from_env() -> Self {
        let brokers = std::env::var("KAFKA_BROKERS")
            .unwrap_or_else(|_| "localhost:9092".to_string());

        Self { brokers }
    }
}