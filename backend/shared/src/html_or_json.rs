use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, StatusCode, header, request::Parts},
    response::{Html, IntoResponse, Response},
};

#[derive(Debug)]
pub enum HtmlOrJson {
    Empty(StatusCode),
    Html(StatusCode, String),
    Json(StatusCode, serde_json::Value),
}

impl IntoResponse for HtmlOrJson {
    fn into_response(self) -> Response {
        match self {
            Self::Empty(status) => status.into_response(),
            Self::Html(status, body) => {
                (status, Html(body)).into_response()
            }
            Self::Json(status, val) => {
                (status, axum::Json(val)).into_response()
            }
        }
    }
}

pub fn wants_html(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").is_some()
        || headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("text/html"))
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy)]
pub enum AcceptFormat {
    Html,
    Json,
}

impl<S> FromRequestParts<S> for AcceptFormat
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if wants_html(&parts.headers) {
            Ok(Self::Html)
        } else {
            Ok(Self::Json)
        }
    }
}