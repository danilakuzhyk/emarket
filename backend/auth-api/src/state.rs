use crate::error::AppError;

#[derive(Clone)]
pub struct AppState {}

impl AppState {
    pub async fn from_env() -> Result<AppState, AppError> {
        Ok(Self {})
    }
}