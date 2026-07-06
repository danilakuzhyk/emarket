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
use crate::ui::{layout, login_form, register_form};
use axum::{
    Router,
    extract::{Form, Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;
use shared::{
    auth::{UserRole, unsafe_decode_role},
    html_or_json::{HtmlOrJson, wants_html},
    kafka_events::{CustomerRegisteredEvent, VendorRegisteredEvent},
};
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

#[tokio::main]
async fn main() {
    let state = AppState::from_env()
        .await
        .expect("failed to initialize application state");

    let app = create_app(state);
    let bind_addr = std::env::var("AUTH_API_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = TcpListener::bind(&bind_addr)
        .await
        .expect("failed to bind tcp listener");
    axum::serve(listener, app)
        .await
        .expect("failed starting server");
}

fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/login", get(Html(layout("e-Market log in", login_form()))))
        .route(
            "/register",
            get(Html(layout("Registration", register_form()))),
        )
        .route("/register/{role-fragment}", get(get_register_fragment))
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
) -> Response {
    let tokens = match login_request(&state.keycloak_state, &payload).await {
        Ok(t) => t,
        Err(e) => return e.into_response_with_headers(&headers),
    };

    let role = unsafe_decode_role(&*tokens.access_token).unwrap_or(UserRole::Customer);
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
            [("HX-Redirect", format!("/{}s/profile", role.to_string()))],
        )
            .into_response();

        (new_jar, response).into_response()
    } else {
        (
            new_jar,
            HtmlOrJson::Json(
                StatusCode::OK,
                serde_json::json!({"status": "authenticated"}),
            ),
        )
            .into_response()
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
        let response = (StatusCode::OK, [("HX-Redirect", "/login")]);
        Ok((new_jar, response).into_response())
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
) -> Response {
    let user_id = match user_register_request(&state.keycloak_state, &payload, "customer").await {
        Ok(id) => id,
        Err(e) => return e.into_response_with_headers(&headers),
    };

    let event = CustomerRegisteredEvent::new(user_id.to_string(), payload.email);
    if let Err(e) = send_customer_registered(&state.kafka_state, event).await {
        return e.into_response_with_headers(&headers);
    };

    if wants_html(&headers) {
        let success_html = ui::html_success_fragment("Success!");

        (
            StatusCode::OK,
            [
                ("HX-Retarget", "#register-card"),
                ("HX-Reswap", "outerHTML"),
            ],
            Html(success_html),
        )
            .into_response()
    } else {
        HtmlOrJson::Json(StatusCode::OK, serde_json::json!({"status": "ok"})).into_response()
    }
}

async fn vendor_register_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(payload): Form<RegisterDTO>,
) -> Response {
    let user_id = match user_register_request(&state.keycloak_state, &payload, "vendor").await {
        Ok(id) => id,
        Err(e) => return e.into_response_with_headers(&headers),
    };

    let event = VendorRegisteredEvent::new(user_id.to_string(), payload.email);
    if let Err(e) = send_vendor_registered(&state.kafka_state, event).await {
        return e.into_response_with_headers(&headers);
    };

    if wants_html(&headers) {
        let success_html = ui::html_success_fragment("Success!");

        (
            StatusCode::OK,
            [
                ("HX-Retarget", "#register-card"),
                ("HX-Reswap", "outerHTML"),
            ],
            Html(success_html),
        )
            .into_response()
    } else {
        HtmlOrJson::Json(StatusCode::OK, serde_json::json!({"status": "ok"})).into_response()
    }
}

async fn get_register_fragment(Path(role): Path<UserRole>) -> impl IntoResponse {
    Html(ui::confirm_button(&role.to_string()))
}
