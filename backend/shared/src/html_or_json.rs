use axum::{
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};

pub enum HtmlOrJson {
    Html(StatusCode, String),
    Json(StatusCode, serde_json::Value),
    Empty(StatusCode),
}

impl IntoResponse for HtmlOrJson {
    fn into_response(self) -> Response {
        match self {
            HtmlOrJson::Html(status, body) => (
                status,
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                body,
            )
                .into_response(),
            HtmlOrJson::Json(status, val) => (
                status,
                [(header::CONTENT_TYPE, "application/json")],
                axum::Json(val),
            )
                .into_response(),
            HtmlOrJson::Empty(status) => status.into_response(),
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
