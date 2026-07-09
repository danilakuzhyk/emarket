#[derive(Clone)]
pub(crate) struct KafkaState {
    bootstrap_server: String,
}

impl KafkaState {
    pub fn new(bootstrap_server: String) -> KafkaState {
        KafkaState { bootstrap_server }
    }
}