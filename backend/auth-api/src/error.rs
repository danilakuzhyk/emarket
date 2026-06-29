use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use reqwest::Error;

pub enum AppError {
    Reqwest(Error),
    Keycloak(&'static str, StatusCode, String),
    Unauthorized,
    Conflict,
    Kafka(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, err_msg) = match self {
            AppError::Reqwest(err) => {
                println!("HTTP Error: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::Keycloak(context, status, text) => {
                println!("Keycloak error during {} [{}]: {}", context, status, text);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Identity provider error".to_string(),
                )
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Conflict => (StatusCode::CONFLICT, "User already exists".to_string()),
            AppError::Kafka(err_msg) => {
                println!("Kafka error {}", err_msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Message broker error".to_string(),
                )
            }
        };
        (status, err_msg).into_response()
    }
}

impl From<Error> for AppError {
    fn from(err: Error) -> Self {
        AppError::Reqwest(err)
    }
}
