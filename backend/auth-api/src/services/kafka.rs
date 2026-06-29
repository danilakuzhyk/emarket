use crate::error::AppError;
use shared::kafka::SharedKafkaProducer;
use shared::kafka_events::{CustomerRegisteredEvent, VendorRegisteredEvent};

#[derive(Clone)]
pub struct KafkaState {
    producer: SharedKafkaProducer,
}

impl Default for KafkaState {
    fn default() -> Self {
        let producer = SharedKafkaProducer::default();
        Self { producer }
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
