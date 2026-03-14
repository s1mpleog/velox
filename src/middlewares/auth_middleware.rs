use axum::{extract::Request, middleware::Next, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{error::VeloxError, services::auth_service::AuthService};

pub async fn auth_middleware(
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, VeloxError> {
    let access_token = jar.get("access_token").ok_or(VeloxError::AuthError)?;
    let access_token_value = access_token.value().to_string();
    let claims = AuthService::verify_access_token(&access_token_value)?;
    req.extensions_mut().insert(claims.user_id);

    tracing::info!("Logged in user id: {}", claims.user_id);

    Ok(next.run(req).await)
}
