use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomerRegisteredEvent {
    pub user_id: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VendorRegisteredEvent {
    pub user_id: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}
