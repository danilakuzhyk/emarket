use crate::error::AuthError;
use shared::kafka::SharedKafkaProducer;

#[derive(Clone)]
pub(crate) struct KafkaState {
    producer: SharedKafkaProducer,
}

impl KafkaState {
    pub fn new(bootstrap_server: url::Url) -> Result<Self, AuthError> {
        let producer = SharedKafkaProducer::new(&bootstrap_server)?;
        Ok(Self { producer })
    }
}