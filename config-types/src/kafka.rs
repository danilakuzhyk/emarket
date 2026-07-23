use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum KafkaConfigError {
    #[error("Bootstrap URL '{0}' is missing a host")]
    MissingHost(Url),

    #[error("Bootstrap URL '{0}' is missing a port")]
    MissingPort(Url),
}

#[derive(Clone, Debug)]
pub struct KafkaBootstrapServer {
    url: Url,
    host: String,
    port: u16,
}

impl KafkaBootstrapServer {
    pub fn new(url: Url) -> Result<Self, KafkaConfigError> {
        let host = url
            .host_str()
            .ok_or_else(|| KafkaConfigError::MissingHost(url.clone()))?
            .to_string();
        let port = url
            .port()
            .ok_or_else(|| KafkaConfigError::MissingPort(url.clone()))?;

        Ok(Self { url, host, port })
    }

    pub fn as_url(&self) -> &Url {
        &self.url
    }

    pub fn as_host_port(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl std::fmt::Display for KafkaBootstrapServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}
