use crate::error::AppError;
use shared::kafka::SharedKafkaProducer;
use shared::kafka_events::{CustomerRegisteredEvent, VendorRegisteredEvent};

#[derive(Clone)]
pub struct KafkaState {
    producer: SharedKafkaProducer,
}

impl KafkaState {
    pub fn from_env() -> Result<Self, AppError> {
        let bootstrap = std::env::var("KAFKA_BOOTSTRAP_SERVERS")
            .unwrap_or_else(|_| "localhost:9092".to_string());
        Ok(Self {
            producer: SharedKafkaProducer::new(&bootstrap),
        })
    }
}

pub async fn send_customer_registered(
    state: &KafkaState,
    event: CustomerRegisteredEvent,
) -> Result<(), AppError> {
    match state
        .producer
        .send_internal("customer-registered", &event.user_id, &event)
        .await
    {
        Ok(_) => Ok(()),
        Err(message) => Err(AppError::Kafka(message)),
    }
}

pub async fn send_vendor_registered(
    state: &KafkaState,
    event: VendorRegisteredEvent,
) -> Result<(), AppError> {
    match state
        .producer
        .send_internal("vendor-registered", &event.user_id, &event)
        .await
    {
        Ok(_) => Ok(()),
        Err(message) => Err(AppError::Kafka(message)),
    }
}