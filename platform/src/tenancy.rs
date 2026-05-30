//! Tenant resolution. Each request must identify its enterprise tenant via the
//! `X-Tenant` header; the value becomes the SurrealDB namespace.

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::error::AppError;

/// The resolved tenant for a request.
pub struct Tenant(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for Tenant
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let raw = parts
            .headers
            .get("x-tenant")
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or(AppError::MissingTenant)?;

        // Namespaces must be a safe identifier; reject anything exotic.
        if !raw
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(AppError::BadRequest(
                "invalid tenant: use [A-Za-z0-9_-]".into(),
            ));
        }
        Ok(Tenant(raw.to_string()))
    }
}
