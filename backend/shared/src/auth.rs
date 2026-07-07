use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use base64::Engine;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::Arc;

use crate::html_or_json::{AcceptFormat, HtmlOrJson};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Customer,
    Vendor,
}

impl Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Customer => write!(f, "customer"),
            UserRole::Vendor => write!(f, "vendor"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: String,
    pub email: String,
    pub role: UserRole,
    pub exp: i64,
}

#[derive(Debug, Deserialize)]
struct RawJwtClaims {
    sub: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    preferred_username: Option<String>,
    #[serde(default)]
    role: Option<UserRole>,
    #[serde(default)]
    realm_access: Option<RealmAccess>,
    exp: i64,
}

#[derive(Debug, Deserialize)]
struct RealmAccess {
    roles: Vec<String>,
}

fn resolve_role(raw: &RawJwtClaims) -> UserRole {
    if let Some(role) = raw.role {
        return role;
    }
    if let Some(access) = &raw.realm_access {
        if access.roles.iter().any(|r| r == "vendor") {
            return UserRole::Vendor;
        }
    }
    UserRole::Customer
}

fn claims_from_raw(raw: RawJwtClaims) -> JwtClaims {
    let email = if raw.email.is_empty() {
        raw.preferred_username.clone().unwrap_or_default()
    } else {
        raw.email.clone()
    };
    let role = resolve_role(&raw);

    JwtClaims {
        sub: raw.sub,
        email,
        role,
        exp: raw.exp,
    }
}

fn decode_jwt_payload(token: &str) -> Option<RawJwtClaims> {
    let payload_b64 = token.split('.').nth(1)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload_b64)
        .ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub fn unsafe_decode_role(token: &str) -> Option<UserRole> {
    decode_jwt_payload(token).map(|raw| resolve_role(&raw))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwkKey {
    pub kid: String,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwksResponse {
    pub keys: Vec<JwkKey>,
}

#[derive(Clone)]
pub struct TokenValidator {
    inner: Arc<TokenValidatorInner>,
}

struct TokenValidatorInner {
    jwks: JwksResponse,
}

impl TokenValidator {
    pub fn new(jwks: JwksResponse) -> Self {
        Self {
            inner: Arc::new(TokenValidatorInner { jwks }),
        }
    }

    pub fn validate_token(&self, token: &str) -> Result<JwtClaims, jsonwebtoken::errors::Error> {
        let header = decode_header(token)?;
        let kid = header.kid.ok_or(jsonwebtoken::errors::ErrorKind::InvalidToken)?;

        let jwk = self
            .inner
            .jwks
            .keys
            .iter()
            .find(|key| key.kid == kid)
            .ok_or(jsonwebtoken::errors::ErrorKind::InvalidToken)?;

        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        validation.validate_aud = false;

        let token_data = decode::<RawJwtClaims>(token, &decoding_key, &validation)?;
        Ok(claims_from_raw(token_data.claims))
    }
}

#[derive(Clone)]
pub struct AuthConfig {
    pub validator: TokenValidator,
    pub http_client: reqwest::Client,
    pub auth_service_base_url: String,
}

#[derive(Clone, Debug)]
pub struct Claims(pub JwtClaims);

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(claims) = parts.extensions.get::<Claims>().cloned() {
            return Ok(claims);
        }

        let format = AcceptFormat::from_request_parts(parts, state)
            .await
            .unwrap_or(AcceptFormat::Json);

        match format {
            AcceptFormat::Html => Err(Redirect::to("/login").into_response()),
            AcceptFormat::Json => Err(HtmlOrJson::Json(
                StatusCode::UNAUTHORIZED,
                serde_json::json!({"error": "Unauthorized"}),
            ).into_response()),
        }
    }
}

async fn refresh_tokens_via_http(
    client: &reqwest::Client,
    refresh_token: &str,
    base_url: &str,
) -> Option<(String, String)> {
    let url = format!("{}/api/users/refresh", base_url.trim_end_matches('/'));

    let response = client
        .post(&url)
        .header(
            reqwest::header::COOKIE,
            format!("refresh_token={}", refresh_token),
        )
        .send()
        .await
        .ok()?;

    if response.status().is_success() {
        let mut access_token = None;
        let mut new_refresh_token = None;

        for cookie in response.cookies() {
            if cookie.name() == "access_token" {
                access_token = Some(cookie.value().to_string());
            }
            if cookie.name() == "refresh_token" {
                new_refresh_token = Some(cookie.value().to_string());
            }
        }

        if let (Some(a), Some(r)) = (access_token, new_refresh_token) {
            return Some((a, r));
        }
    }
    None
}

pub async fn auth_middleware(
    State(config): State<AuthConfig>,
    format: AcceptFormat,
    jar: CookieJar,
    mut req: axum::extract::Request<Body>,
    next: Next,
) -> Response {
    if let Some(token) = jar.get("access_token").map(|c| c.value()) {
        if let Ok(claims) = config.validator.validate_token(token) {
            req.extensions_mut().insert(Claims(claims));
            return next.run(req).await;
        }
    }

    if let Some(ref_token) = jar.get("refresh_token").map(|c| c.value()) {
        if let Some((new_access, new_refresh)) =
            refresh_tokens_via_http(&config.http_client, ref_token, &config.auth_service_base_url).await
        {
            if let Ok(claims) = config.validator.validate_token(&new_access) {
                req.extensions_mut().insert(Claims(claims));

                let response = next.run(req).await;

                let response_jar = jar
                    .add(Cookie::build(("access_token", new_access)).path("/").http_only(true))
                    .add(Cookie::build(("refresh_token", new_refresh)).path("/").http_only(true));

                return (response_jar, response).into_response();
            }
        }
    }

    match format {
        AcceptFormat::Html => Redirect::to("/login").into_response(),
        AcceptFormat::Json => HtmlOrJson::Json(
            StatusCode::UNAUTHORIZED,
            serde_json::json!({"status": "unauthorized"}),
        ).into_response()
    }
}