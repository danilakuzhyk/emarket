mod services;
mod error;
mod ui;
mod state;

use crate::error::AppError;
use crate::services::keycloak::{
    forgot_password_request, login_request, logout_request, refresh_request, user_register_request,
};
use crate::state::AppState;
use crate::ui::{forgot_password_form, layout, login_form, register_form};
use axum::{
    Router,
    serve,
    response::{Html},
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::Deserialize;
use shared::{
    auth::{UserRole, unsafe_decode_role},
    html_or_json::{AcceptFormat, HtmlOrJson},
};
use tokio::net::TcpListener;
use state::AppState;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginDTO {
    login: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterDTO {
    first_name: String,
    second_name: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct ForgotPasswordDTO {
    email: String,
}

#[tokio::main]
async fn main() {
    let state = AppState::from_env()
        .await
        .expect("failed to initialize application state");
    let app = create_app(state);
    let bind_adr = std::env::var("BIND_ADRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = TcpListener::bind(&bind_adr).await.expect("failed to bind listener");
    serve(listener, app).await.expect("failed to start server");
}

fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/login", get(|| async { Html(layout("e-Market log in", login_form()))}))
        .route("/register", get(|| async { Html(layout("Registration", register_form()))}))
        .route("/forgot-password", get(|| async { Html(layout("Forgot password", forgot_password_form()))}))
        .route("/register/fragment/{role}", get(get_register_fragment))
        .route("/api/users/certs", get(get_public_certs_handler))
        .route("/api/users/login", post(login_handler))
        .route("/api/users/logout", post(logout_handler))
        .route("/api/users/refresh", post(refresh_handler))
        .route("/api/users/forgot-password", post(forgot_password_handler))
        .route(
            "/api/users/customers/register",
            post(customer_register_handler),
        )
        .route("/api/users/vendors/register", post(vendor_register_handler))
        .with_state(state)
}

async fn customer_register_handler(
    State(state): State<AppState>,
    format: AcceptFormat,
    Form(payload): Form<RegisterDTO>,
) -> Result<Response, crate::error::FormattedAppError> {
    let user_id = user_register_request(&state.keycloak_state, &payload, "customer")
        .await
        .map_err(|e| e.with_format(format))?;

    match format {
        AcceptFormat::Html => {
            let success_html = ui::html_success_fragment("Success!");
            Ok((
                StatusCode::OK,
                [
                    ("HX-Retarget", "#register-card"),
                    ("HX-Reswap", "outerHTML"),
                ],
                Html(success_html),
            ).into_response())
        }
        AcceptFormat::Json => {
            Ok(HtmlOrJson::Json(StatusCode::OK, serde_json::json!({"status": "ok"})).into_response())
        }
    }
}

async fn vendor_register_handler(
    State(state): State<AppState>,
    format: AcceptFormat,
    Form(payload): Form<RegisterDTO>,
) -> Result<Response, crate::error::FormattedAppError> {
    let user_id = user_register_request(&state.keycloak_state, &payload, "vendor")
        .await
        .map_err(|e| e.with_format(format))?;

    match format {
        AcceptFormat::Html => {
            let success_html = ui::html_success_fragment("Success!");
            Ok((
                StatusCode::OK,
                [
                    ("HX-Retarget", "#register-card"),
                    ("HX-Reswap", "outerHTML"),
                ],
                Html(success_html),
            ).into_response())
        }
        AcceptFormat::Json => {
            Ok(HtmlOrJson::Json(StatusCode::OK, serde_json::json!({"status": "ok"})).into_response())
        }
    }
}

async fn get_register_fragment(Path(role): Path<UserRole>) -> impl IntoResponse {
    Html(ui::confirm_button(&role.to_string()))
}