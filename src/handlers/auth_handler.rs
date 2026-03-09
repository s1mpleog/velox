use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use cookie::Cookie;

use crate::{app_state::AppState, dto, error::VeloxError, services::auth_service::AuthService};

use validator::Validate;

pub struct AuthHandler {}

impl AuthHandler {
    pub async fn login_user(
        State(state): State<AppState>,
        Json(body): Json<dto::auth_dto::LoginRequest>,
    ) -> Result<impl IntoResponse, VeloxError> {
        body.validate()
            .map_err(|e| VeloxError::ValidationError(e.to_string()))?;

        AuthService::login(&state.pool, &body.email).await?;

        // tracing::debug!("{:?}", login.err());

        Ok((StatusCode::OK, "user logged in successfully"))
    }

    pub async fn authorize(
        Query(params): Query<dto::auth_dto::AuthorizeQuery>,
        State(state): State<AppState>,
        jar: CookieJar,
    ) -> Result<(CookieJar, impl IntoResponse), VeloxError> {
        tracing::info!("got token as: {}", params.token);

        let (access_token, refresh_token) =
            AuthService::authorize(&state.pool, &params.token).await?;

        // create cookie for access_token and refresh_token

        // TODO: instead of hardcoding add something like access_token_expires and
        // refresh_token_expires inside .env
        let access_token_expires = cookie::time::Duration::minutes(15);

        let access_token_cookie = Cookie::build(("access_token", access_token))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(cookie::SameSite::Lax)
            .max_age(access_token_expires)
            .build();

        let refresh_token_expires = cookie::time::Duration::days(30);

        let refresh_token_cookie = Cookie::build(("refresh_token", refresh_token))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(cookie::SameSite::Lax)
            .max_age(refresh_token_expires)
            .build();

        let jar = jar.add(access_token_cookie).add(refresh_token_cookie);

        Ok((jar, (StatusCode::OK, "authorized successfully")))
    }
}
