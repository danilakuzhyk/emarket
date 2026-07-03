use crate::error::AppError;
use crate::{LoginDTO, RegisterDTO};
use axum::http::StatusCode;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone)]
pub struct KeycloakState {
    http_client: Client,
    keycloak_base_url: String,
    keycloak_realm: String,
    keycloak_client_id: String,
    keycloak_client_secret: String,
}
impl Default for KeycloakState {
    fn default() -> Self {
        Self {
            http_client: Client::new(),
            keycloak_base_url: "http://localhost:8080".to_string(),
            keycloak_realm: "emarket".to_string(),
            keycloak_client_id: "emarket-app".to_string(),
            keycloak_client_secret: "nqLfnsT8VqSoB4Pl3Wq6c7QtznfayvDf".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserID(String);

impl Display for UserID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize)]
pub(crate) struct KeycloakTokenResponse {
    pub(crate) access_token: String,
    pub(crate) refresh_token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct KeycloakUserRequest {
    username: String,
    email: String,
    enabled: bool,
    credentials: Vec<KeycloakCredential>,
    first_name: Option<String>,
    last_name: Option<String>,
}

#[derive(Serialize)]
struct KeycloakCredential {
    #[serde(rename = "type")]
    credential_type: String,
    value: String,
    temporary: bool,
}

#[derive(Deserialize)]
struct AdminTokenResponse {
    access_token: String,
}

pub async fn login_request(
    state: &KeycloakState,
    payload: &LoginDTO,
) -> Result<KeycloakTokenResponse, AppError> {
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

    let response = state.http_client.post(&url).form(&params).send().await?;

    if response.status().is_success() {
        let tokens: KeycloakTokenResponse = response.json().await?;
        Ok(tokens)
    } else {
        Err(AppError::Unauthorized)
    }
}

pub async fn logout_request(state: &KeycloakState, refresh_token: &str) -> Result<(), AppError> {
    let url = format!(
        "{}/realms/{}/protocol/openid-connect/logout",
        state.keycloak_base_url, state.keycloak_realm
    );
    let params = [
        ("client_id", state.keycloak_client_id.as_str()),
        ("client_secret", state.keycloak_client_secret.as_str()),
        ("refresh_token", refresh_token),
    ];

    let response = state.http_client.post(&url).form(&params).send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(AppError::Keycloak(
            "logout",
            response.status(),
            response.text().await.unwrap_or_default(),
        ))
    }
}

pub async fn refresh_request(
    state: &KeycloakState,
    old_refresh_token: &str,
) -> Result<KeycloakTokenResponse, AppError> {
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

    let response = state.http_client.post(&url).form(&params).send().await?;

    if response.status().is_success() {
        let tokens: KeycloakTokenResponse = response.json().await?;
        Ok(tokens)
    } else {
        Err(AppError::Unauthorized)
    }
}

pub async fn user_register_request(
    state: &KeycloakState,
    payload: &RegisterDTO,
    role: &str,
) -> Result<UserID, AppError> {
    println!("Регистрация пользователя. Переданная роль: {}", role);
    let admin_token = get_admin_token(&state).await?;
    let user_id = register_new_user(state, payload, &*admin_token).await?;
    set_user_role(&state, &*admin_token, &user_id, role).await?;
    Ok(user_id)
}

pub async fn set_user_role(
    state: &KeycloakState,
    admin_token: &str,
    user_id: &UserID,
    role: &str,
) -> Result<(), AppError> {
    let role = get_role_by_name(&state, &admin_token, role).await?;
    let role_url = format!(
        "{}/admin/realms/{}/users/{}/role-mappings/realm",
        state.keycloak_base_url, state.keycloak_realm, user_id
    );
    let response = state
        .http_client
        .post(&role_url)
        .bearer_auth(&admin_token)
        .json(&vec![role])
        .send()
        .await?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(AppError::Keycloak(
            "role mapping",
            response.status(),
            response.text().await.unwrap_or_default(),
        ))
    }
}

async fn get_admin_token(state: &KeycloakState) -> Result<String, AppError> {
    let url = format!(
        "{}/realms/{}/protocol/openid-connect/token",
        state.keycloak_base_url, state.keycloak_realm
    );
    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", &state.keycloak_client_id),
        ("client_secret", &state.keycloak_client_secret),
    ];

    let response = state.http_client.post(&url).form(&params).send().await?;

    if response.status().is_success() {
        let json: AdminTokenResponse = response.json().await?;
        Ok(json.access_token)
    } else {
        Err(AppError::Keycloak(
            "admin token fetching",
            response.status(),
            response.text().await.unwrap_or_default(),
        ))
    }
}

async fn get_role_by_name(
    state: &KeycloakState,
    admin_token: &str,
    role_name: &str,
) -> Result<serde_json::Value, AppError> {
    let url = format!(
        "{}/admin/realms/{}/roles/{}",
        state.keycloak_base_url, state.keycloak_realm, role_name
    );

    let response = state
        .http_client
        .get(&url)
        .bearer_auth(admin_token)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(response.json::<serde_json::Value>().await?)
    } else {
        Err(AppError::Keycloak(
            "role fetching",
            response.status(),
            response.text().await.unwrap_or_default(),
        ))
    }
}

async fn register_new_user(
    state: &KeycloakState,
    payload: &RegisterDTO,
    admin_token: &str,
) -> Result<UserID, AppError> {
    let register_url = format!(
        "{}/admin/realms/{}/users",
        state.keycloak_base_url, state.keycloak_realm
    );

    let new_user = KeycloakUserRequest {
        username: payload.email.clone(),
        email: payload.email.clone(),
        enabled: true,
        first_name: Some(payload.first_name.clone()),
        last_name: Some(payload.second_name.clone()),
        credentials: vec![KeycloakCredential {
            credential_type: "password".to_string(),
            value: payload.password.clone(),
            temporary: false,
        }],
    };

    let response = state
        .http_client
        .post(&register_url)
        .bearer_auth(admin_token)
        .json(&new_user)
        .send()
        .await?;

    let status = response.status();

    if status == StatusCode::CREATED {
        if let Some(location) = response.headers().get("Location") {
            if let Ok(location_str) = location.to_str() {
                if let Some(uuid) = location_str.split('/').last() {
                    return Ok(UserID(uuid.to_string()));
                }
            }
        }
        Err(AppError::Keycloak(
            "user creation",
            StatusCode::INTERNAL_SERVER_ERROR,
            "No UUID in Location header".to_string(),
        ))
    } else if status == StatusCode::CONFLICT {
        Err(AppError::Conflict)
    } else {
        Err(AppError::Keycloak(
            "user creation",
            status,
            response.text().await.unwrap_or_default(),
        ))
    }
}
