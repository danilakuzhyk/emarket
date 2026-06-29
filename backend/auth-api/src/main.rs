mod error;
mod services;
mod state;
mod ui;

use crate::error::AppError;
use crate::services::kafka::{send_customer_registered, send_vendor_registered};
use crate::services::keycloak::{
    login_request, logout_request, refresh_request, user_register_request,
};
use crate::state::AppState;
use crate::ui::{
    customer_register_form, html_success_fragment, layout, login_form, vendor_register_form,
};
use axum::{
    Router,
    extract::{Form, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;
use shared::kafka_events::{CustomerRegisteredEvent, VendorRegisteredEvent};
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct LoginDTO {
    login: String,
    password: String,
}

#[derive(Deserialize)]
struct RegisterDTO {
    first_name: String,
    second_name: String,
    email: String,
    password: String,
}

enum HtmlOrJson {
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

fn wants_html(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").is_some()
        || headers
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("text/html"))
            .unwrap_or(false)
}

#[tokio::main]
async fn main() {
    let app = create_app(AppState::default());
    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind tcp listener");
    axum::serve(listener, app)
        .await
        .expect("failed starting server");
}

fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/login", get(get_login_page))
        .route("/register/customer", get(get_customer_register_page))
        .route("/register/vendor", get(get_vendor_register_page))
        .route("/api/users/login", post(login_handler))
        .route("/api/users/logout", post(logout_handler))
        .route("/api/users/refresh", post(refresh_handler))
        .route(
            "/api/users/customers/register",
            post(customer_register_handler),
        )
        .route("/api/users/vendors/register", post(vendor_register_handler))
        .with_state(state)
}

async fn login_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Form(payload): Form<LoginDTO>,
) -> Result<Response, AppError> {
    let tokens = login_request(&state.keycloak_state, &payload).await?;
    let new_jar = jar
        .add(
            Cookie::build(("access_token", tokens.access_token))
                .path("/")
                .http_only(true),
        )
        .add(
            Cookie::build(("refresh_token", tokens.refresh_token))
                .path("/")
                .http_only(true),
        );

    if wants_html(&headers) {
        let response = (
            StatusCode::OK,
            [("HX-Redirect", "/api/customers/profile")], //TODO: choose where to go
        )
            .into_response();

        Ok((new_jar, response).into_response())
    } else {
        Ok((
            new_jar,
            HtmlOrJson::Json(
                StatusCode::OK,
                serde_json::json!({"status": "authenticated"}),
            ),
        )
            .into_response())
    }
}

async fn logout_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let refresh_token = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;
    logout_request(&state.keycloak_state, &refresh_token).await?;
    let new_jar = jar
        .remove(Cookie::from("access_token"))
        .remove(Cookie::from("refresh_token"));

    if wants_html(&headers) {
        let html = String::new(); //TODO
        Ok((new_jar, HtmlOrJson::Html(StatusCode::OK, html)).into_response())
    } else {
        Ok((
            new_jar,
            HtmlOrJson::Json(StatusCode::OK, serde_json::json!({"status": "ok"})),
        )
            .into_response())
    }
}

async fn refresh_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let old_refresh_token = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    let tokens = refresh_request(&state.keycloak_state, &old_refresh_token).await?;
    let new_jar = jar
        .add(
            Cookie::build(("access_token", tokens.access_token))
                .path("/")
                .http_only(true),
        )
        .add(
            Cookie::build(("refresh_token", tokens.refresh_token))
                .path("/")
                .http_only(true),
        );

    if wants_html(&headers) {
        Ok((new_jar, HtmlOrJson::Empty(StatusCode::OK)).into_response())
    } else {
        Ok((
            new_jar,
            HtmlOrJson::Json(StatusCode::OK, serde_json::json!({"status": "ok"})),
        )
            .into_response())
    }
}

async fn customer_register_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(payload): Form<RegisterDTO>,
) -> Result<Response, AppError> {
    let user_id = user_register_request(&state.keycloak_state, &payload, "customer").await?;
    let event = CustomerRegisteredEvent::new(user_id, payload.email);
    send_customer_registered(&state.kafka_state, event).await?;
    if wants_html(&headers) {
        let html = html_success_fragment(""); // TODO: make up message
        Ok(HtmlOrJson::Html(StatusCode::CREATED, html).into_response())
    } else {
        Ok(
            HtmlOrJson::Json(StatusCode::CREATED, serde_json::json!({"status": "ok"}))
                .into_response(),
        )
    }
}

async fn vendor_register_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(payload): Form<RegisterDTO>,
) -> Result<Response, AppError> {
    let user_id = user_register_request(&state.keycloak_state, &payload, "vendor").await?;
    let event = VendorRegisteredEvent::new(user_id, payload.email);
    send_vendor_registered(&state.kafka_state, event).await?;
    if wants_html(&headers) {
        let html = html_success_fragment(""); //TODO: make up message
        Ok(HtmlOrJson::Html(StatusCode::CREATED, html).into_response())
    } else {
        Ok(
            HtmlOrJson::Json(StatusCode::CREATED, serde_json::json!({"status": "ok"}))
                .into_response(),
        )
    }
}

async fn get_login_page() -> Html<String> {
    Html(layout("e-Market log in", login_form()))
}

async fn get_customer_register_page() -> Html<String> {
    Html(layout("Customer registration", customer_register_form()))
}

async fn get_vendor_register_page() -> Html<String> {
    Html(layout("Vendor registration", vendor_register_form()))
}
