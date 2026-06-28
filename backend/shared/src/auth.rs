use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    Customer,
    Vendor,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: String,
    pub email: String,
    pub role: UserRole,
    pub exp: i64,
}
