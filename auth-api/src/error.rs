use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum AuthInitError {
    #[error("Failed to initialize Kafka producer: {0}")]
    Kafka(#[from] shared::kafka::KafkaClientError),
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Internal message broker error: {0}")]
    KafkaError(#[from] shared::kafka::KafkaClientError),

    #[error("Identity provider error: {0}")]
    KeycloakRequestError(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::UserAlreadyExists => (StatusCode::CONFLICT, self.to_string()),
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::KafkaError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AuthError::KeycloakRequestError(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
        };

        let body = Json(serde_json::json!({ "error": error_message }));
        (status, body).into_response()
    }
}
