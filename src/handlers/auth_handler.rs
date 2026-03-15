use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use cookie::{Cookie, SameSite};

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

        tracing::info!("login body validated");

        AuthService::login(&state.pool, &body.email).await?;

        tracing::info!("auth service ran successfully");

        // tracing::debug!("{:?}", login.err());

        Ok((StatusCode::OK, "magic link sent to your email"))
    }

    pub async fn authorize(
        Query(params): Query<dto::auth_dto::AuthorizeQuery>,
        State(state): State<AppState>,
        jar: CookieJar,
    ) -> Result<(CookieJar, impl IntoResponse), VeloxError> {
        tracing::info!("received token");

        let (access_token, refresh_token) =
            AuthService::authorize(&state.pool, &params.token).await?;

        tracing::info!("Sucessfully ran authorize service, created refresh and access token");

        // create cookie for access_token and refresh_token

        // TODO: instead of hardcoding add something like access_token_expires and
        // refresh_token_expires inside .env
        let access_token_expires = cookie::time::Duration::minutes(15);

        let access_token_cookie = Cookie::build(("access_token", access_token))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(cookie::SameSite::Strict)
            .max_age(access_token_expires)
            .build();

        let refresh_token_expires = cookie::time::Duration::days(30);

        let refresh_token_cookie = Cookie::build(("refresh_token", refresh_token))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(cookie::SameSite::Strict)
            .max_age(refresh_token_expires)
            .build();

        let jar = jar.add(access_token_cookie).add(refresh_token_cookie);

        tracing::info!("successfully authorized user");

        Ok((jar, (StatusCode::OK, "authorized successfully")))
    }

    pub async fn refresh(
        State(state): State<AppState>,
        jar: CookieJar,
    ) -> Result<(CookieJar, impl IntoResponse), VeloxError> {
        let refresh_token = jar.get("refresh_token").ok_or(VeloxError::AuthError)?;
        let refresh_token_to_string = refresh_token.value().to_string();

        tracing::info!("fetched refresh token successfully");

        let (access_token, refresh_token) =
            AuthService::refresh(&state.pool, &refresh_token_to_string).await?;

        tracing::info!("refresh service ran successfully generated new access token");

        let access_token_expires = cookie::time::Duration::minutes(15);

        let access_token_cookie = Cookie::build(("access_token", access_token))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(cookie::SameSite::Strict)
            .max_age(access_token_expires)
            .build();

        let refresh_token_expires = cookie::time::Duration::days(30);

        let refresh_token_cookie = Cookie::build(("refresh_token", refresh_token))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(cookie::SameSite::Strict)
            .max_age(refresh_token_expires)
            .build();

        let jar = jar.add(access_token_cookie).add(refresh_token_cookie);

        tracing::info!("created cookie successfully");

        Ok((jar, (StatusCode::OK, "successfully generated access_token")))
    }

    pub async fn logout(
        State(state): State<AppState>,
        jar: CookieJar,
    ) -> Result<(CookieJar, impl IntoResponse), VeloxError> {
        let refresh_token = jar.get("refresh_token").ok_or(VeloxError::AuthError)?;
        let refresh_token_to_string = refresh_token.value().to_string();

        tracing::info!("refresh token: {refresh_token_to_string}");

        tracing::info!("fetched refresh token successfully");

        AuthService::logout(&state.pool, &refresh_token_to_string).await?;

        tracing::info!("logout service ran successfully");

        let removal_cookie = Cookie::build(Cookie::from("refresh_token"))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict);

        let removal_access = Cookie::build(Cookie::from("access_token"))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict);

        let jar = jar.remove(removal_cookie).remove(removal_access);

        // tracing::debug!("{:?}", jar);

        tracing::info!("deleted cookies successfully");

        Ok((jar, (StatusCode::OK, "user logged out successfully")))
    }
}
