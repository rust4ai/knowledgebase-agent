use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::error::AppError;
use crate::state::AppState;

/// Extractor that validates `Authorization: Bearer <ADMIN_PASSWORD>`.
/// Add this to any handler that requires admin access.
pub struct AdminAuth;

impl FromRequestParts<AppState> for AdminAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let token = header.strip_prefix("Bearer ").unwrap_or("");

        if token.is_empty() || token != state.config.admin_password {
            return Err(AppError::Unauthorized);
        }

        Ok(AdminAuth)
    }
}
