use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomerRegisteredEvent {
    pub user_id: String,
    pub email: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VendorRegisteredEvent {
    pub user_id: String,
    pub email: String,
    pub created_at: OffsetDateTime,
}

impl CustomerRegisteredEvent {
    pub fn new(user_id: String, email: String) -> Self {
        Self {
            user_id,
            email,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

impl VendorRegisteredEvent {
    pub fn new(user_id: String, email: String) -> Self {
        Self {
            user_id,
            email,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}
