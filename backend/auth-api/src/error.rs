use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use shared::html_or_json::AcceptFormat;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Conflict: User already exists")]
    Conflict,

    #[error("Kafka Error: {0}")]
    Kafka(String),

    #[error("Keycloak Error ({0}) [{1}]: {2}")]
    Keycloak(&'static str, StatusCode, String),

    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Unauthorized: Invalid credentials")]
    Unauthorized,
}

impl AppError {
    fn to_status_and_message(&self) -> (StatusCode, String) {
        match self {
            Self::Conflict => (
                StatusCode::CONFLICT,
                "User with this email already exists".to_string(),
            ),
            Self::Kafka(err_msg) => {
                println!("Kafka error: {}", err_msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Message broker error".to_string(),
                )
            }
            Self::Keycloak(context, status, text) => {
                println!("Keycloak error during {} [{}]: {}", context, status, text);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Identity provider error".to_string(),
                )
            }
            Self::Reqwest(err) => {
                println!("HTTP Client Error: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Invalid email or password".to_string(),
            ),
        }
    }

    pub fn with_format(self, format: AcceptFormat) -> FormattedAppError {
        FormattedAppError { error: self, format }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, err_msg) = self.to_status_and_message();
        (status, err_msg).into_response()
    }
}

pub struct FormattedAppError {
    pub error: AppError,
    pub format: AcceptFormat,
}

impl IntoResponse for FormattedAppError {
    fn into_response(self) -> Response {
        let (status, err_msg) = self.error.to_status_and_message();

        match self.format {
            AcceptFormat::Html => {
                let html_fragment = crate::ui::html_error_fragment(&err_msg);
                (StatusCode::OK, Html(html_fragment)).into_response()
            }
            AcceptFormat::Json => {
                (status, err_msg).into_response()
            }
        }
    }
}