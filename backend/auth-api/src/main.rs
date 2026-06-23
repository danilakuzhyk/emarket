use axum::{
    Router,
    extract::{Form, State},
    response::IntoResponse,
    routing::post,
    serve,
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::ops::Add;
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    http_client: reqwest::Client,
    keycloak_base_url: String,
    keycloak_realm: String,
    keycloak_client_id: String,
    keycloak_client_secret: String,
}

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

#[derive(Deserialize)]
struct KeycloakTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64, //TODO: use
}

#[derive(Serialize)]
struct KeycloakUserRequest {
    username: String,
    email: String,
    enabled: bool,
    credentials: Vec<KeycloakCredential>,
}

#[derive(Serialize)]
struct KeycloakCredential {
    #[serde(rename = "type")]
    credential_type: String,
    value: String,
    temporary: bool,
}

#[derive(Serialize)]
struct KeycloakRole {
    name: String,
}

#[derive(Deserialize)]
struct AdminTokenResponse {
    access_token: String,
}

#[tokio::main]
async fn main() {
    let app = create_app(AppState {
        http_client: reqwest::Client::new(),
        keycloak_base_url: "http://localhost:8080".to_string(),
        keycloak_realm: "emarket".to_string(),
        keycloak_client_id: "emarket-app".to_string(),
        keycloak_client_secret: "9F0z3yzlEGeqdIWvyKcSKHhwZOnMAxwA".to_string(),
    });
    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind tcp listener"); //TODO choose a port
    serve(listener, app).await.expect("failed starting server"); //TODO
}

/// ## Endpoints
/// - `/api/users/login` - authorizes user and returns JWT and refresh token ([login_handler])
/// - `/api/users/logout` - logout user. ([logout_handler])
/// - `/api/users/refresh` - refreshes user's JWT token with refresh token.([refresh_handler])
/// - `/api/users/customers/register` - registers user in keycloak and publishes message to kafka topic 'customer-registered', that customer created an account([customer_register_handler])
/// - `/api/users/vendors/register` - registers user in keycloak and publishes message to kafka topic 'vendor-registered', that vendor created an account([vendor_register_handler])
fn create_app(state: AppState) -> Router {
    Router::new()
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
    Form(payload): Form<LoginDTO>,
) -> impl IntoResponse {
    let url = format!(
        "{}/realms/{}/protocol/openid-connect/token",
        state.keycloak_base_url, state.keycloak_realm
    );
    let params = [
        ("grant_type", "password"),
        ("client_id", &state.keycloak_client_id),
        ("client_secret", &state.keycloak_client_secret),
        ("username", &payload.login),
        ("password", &payload.password),
    ];
    let response = match state.http_client.post(&url).form(&params).send().await {
        Ok(response) => response,
        Err(_) => {
            return (jar, StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };

    if response.status().is_success() {
        let tokens: KeycloakTokenResponse = match response.json().await {
            Ok(json) => json,
            Err(_) => {
                return (jar, StatusCode::INTERNAL_SERVER_ERROR).into_response();
            }
        };
        let access_cookie = Cookie::build(("access_token", tokens.access_token))
            .path("/")
            .http_only(true);
        let refresh_cookie = Cookie::build(("refresh_token", tokens.refresh_token))
            .path("/")
            .http_only(true);
        let new_jar = jar.add(access_cookie).add(refresh_cookie);
        (new_jar, StatusCode::OK).into_response()
    } else {
        (jar, StatusCode::INTERNAL_SERVER_ERROR).into_response()
    }
}

async fn logout_handler(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let refresh_token = match jar.get("refresh_token") {
        Some(cookie) => cookie.value().to_string(),
        None => return (jar, StatusCode::UNAUTHORIZED).into_response(), //TODO json response to html
    };

    let url = format!(
        "{}/realms/{}/protocol/openid-connect/logout",
        state.keycloak_base_url, state.keycloak_realm
    );
    let params = [
        ("client_id", &state.keycloak_client_id),
        ("client_secret", &state.keycloak_client_secret),
        ("refresh_token", &refresh_token),
    ];

    let response = match state.http_client.post(&url).form(&params).send().await {
        Ok(response) => response,
        Err(_) => {
            return (jar, StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };

    if response.status().is_success() {
        let new_jar = jar
            .remove(Cookie::from("access_token"))
            .remove(Cookie::from("refresh_token"));
        (new_jar, StatusCode::OK).into_response()
    } else {
        (jar, StatusCode::INTERNAL_SERVER_ERROR).into_response()
    }
}

async fn refresh_handler(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let old_refresh_token = match jar.get("refresh_token") {
        Some(cookie) => cookie.value().to_string(),
        None => return (jar, StatusCode::UNAUTHORIZED).into_response(), //TODO json response to html
    };

    let url = format!(
        "{}/realms/{}/protocol/openid-connect/token",
        state.keycloak_base_url, state.keycloak_realm
    );
    let params = [
        ("grant_type", "refresh_token"),
        ("client_id", &state.keycloak_client_id),
        ("client_secret", &state.keycloak_client_secret),
        ("refresh_token", &old_refresh_token),
    ];

    let response = match state.http_client.post(&url).form(&params).send().await {
        Ok(response) => response,
        Err(_) => {
            return (jar, StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };
    if response.status().is_success() {
        let tokens: KeycloakTokenResponse = match response.json().await {
            Ok(json) => json,
            Err(_) => {
                return (jar, StatusCode::INTERNAL_SERVER_ERROR).into_response();
            }
        };
        let access_cookie = Cookie::build(("access_token", tokens.access_token))
            .path("/")
            .http_only(true);
        let refresh_cookie = Cookie::build(("refresh_token", tokens.refresh_token))
            .path("/")
            .http_only(true);
        let new_jar = jar.add(access_cookie).add(refresh_cookie);
        (new_jar, StatusCode::OK).into_response()
    } else {
        (jar, StatusCode::UNAUTHORIZED).into_response()
    }
}

async fn customer_register_handler(
    State(state): State<AppState>,
    Form(payload): Form<RegisterDTO>,
) -> impl IntoResponse {
    let user_id = match register_new_user(state.clone(), payload).await {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };

    let admin_token = match get_admin_token(&state).await {
        Ok(token) => token,
        Err(status) => return status.into_response(),
    };

    let role_url = format!(
        "{}/admin/realms/{}/users/{}/role-mappings/realm",
        state.keycloak_base_url, state.keycloak_realm, user_id
    );

    let roles_payload = vec![KeycloakRole {
        name: "customer".to_string(),
    }];

    let response = match state
        .http_client
        .post(&role_url)
        .bearer_auth(admin_token)
        .json(&roles_payload)
        .send()
        .await
    {
        Ok(res) => res,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if response.status().is_success() {
        // TODO: 'customer-registered'
        StatusCode::CREATED.into_response()
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

async fn vendor_register_handler(
    State(state): State<AppState>,
    Form(payload): Form<RegisterDTO>,
) -> impl IntoResponse {
    let user_id = match register_new_user(state.clone(), payload).await {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };

    let admin_token = match get_admin_token(&state).await {
        Ok(token) => token,
        Err(status) => return status.into_response(),
    };

    let role_url = format!(
        "{}/admin/realms/{}/users/{}/role-mappings/realm",
        state.keycloak_base_url, state.keycloak_realm, user_id
    );

    let roles_payload = vec![KeycloakRole {
        name: "vendor".to_string(),
    }];

    let response = match state
        .http_client
        .post(&role_url)
        .bearer_auth(admin_token)
        .json(&roles_payload)
        .send()
        .await
    {
        Ok(res) => res,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if response.status().is_success() {
        // TODO: kafka 'vendor-registered'
        StatusCode::CREATED.into_response()
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

async fn get_admin_token(state: &AppState) -> Result<String, StatusCode> {
    let url = format!(
        "{}/realms/{}/protocol/openid-connect/token",
        state.keycloak_base_url, state.keycloak_realm
    );
    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", &state.keycloak_client_id),
        ("client_secret", &state.keycloak_client_secret),
    ];
    let response = state
        .http_client
        .post(&url)
        .form(&params)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if response.status().is_success() {
        let json: AdminTokenResponse = response
            .json()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(json.access_token)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn register_new_user(state: AppState, payload: RegisterDTO) -> Result<String, StatusCode> {
    let admin_token = get_admin_token(&state).await?;

    let register_url = format!(
        "{}/admin/realms/{}/users",
        state.keycloak_base_url, state.keycloak_realm
    );

    let new_user = KeycloakUserRequest {
        username: payload.first_name.add(" ").add(&payload.second_name),
        email: payload.email,
        enabled: true,
        credentials: vec![KeycloakCredential {
            credential_type: "password".to_string(),
            value: payload.password,
            temporary: false,
        }],
    };

    let response = state
        .http_client
        .post(&register_url)
        .bearer_auth(admin_token)
        .json(&new_user)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if response.status() == StatusCode::CREATED {
        if let Some(location) = response.headers().get("Location") {
            if let Ok(location_str) = location.to_str() {
                if let Some(uuid) = location_str.split('/').last() {
                    return Ok(uuid.to_string());
                }
            }
        }
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    } else if response.status() == StatusCode::CONFLICT {
        Err(StatusCode::CONFLICT)
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}
