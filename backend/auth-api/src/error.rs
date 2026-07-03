use axum::{
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};

pub enum AppError {
    Reqwest(reqwest::Error),
    Keycloak(&'static str, StatusCode, String),
    Unauthorized,
    Conflict,
    Kafka(String),
}

impl AppError {
    pub fn into_response_with_headers(self, headers: &HeaderMap) -> Response {
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
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Invalid email or password".to_string(),
            ),
            AppError::Conflict => (
                StatusCode::CONFLICT,
                "User with this email already exists".to_string(),
            ),
            AppError::Kafka(err_msg) => {
                println!("Kafka error {}", err_msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Message broker error".to_string(),
                )
            }
        };

        if shared::html_or_json::wants_html(headers) {
            let html_fragment = crate::ui::html_error_fragment(&err_msg);

            (StatusCode::OK, Html(html_fragment)).into_response()
        } else {
            (status, err_msg).into_response()
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let headers = HeaderMap::new();
        self.into_response_with_headers(&headers)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Reqwest(err)
    }
}
