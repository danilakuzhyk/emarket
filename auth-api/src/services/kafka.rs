use config_types::kafka::KafkaBootstrapServer;
use shared::kafka::{KafkaClientError, SharedKafkaProducer};

#[derive(Clone)]
pub(crate) struct KafkaState {
    producer: SharedKafkaProducer,
}

impl KafkaState {
    pub fn new(bootstrap_server: &KafkaBootstrapServer) -> Result<Self, KafkaClientError> {
        let producer = SharedKafkaProducer::new(bootstrap_server)?;
        Ok(Self { producer })
    }
}
