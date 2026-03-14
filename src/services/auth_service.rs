use rand::{Rng, rng};
use resend_rs::types::CreateEmailBaseOptions;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, Pool, Postgres};

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use uuid::Uuid;

use crate::error::VeloxError;
use crate::models::magic_token_model::ResponseType;
use crate::repositories::magic_token_repository::MagicTokenRepository;
use crate::repositories::refresh_token_repository::RefreshTokenRepository;
use crate::repositories::temp_user_repository::TempUserRepository;
use crate::repositories::user_repository::UserRepository;
use crate::utils::Utils;

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub user_id: Uuid,
    pub exp: usize,
}

pub struct AuthService {}

impl AuthService {
    fn generate_random_token() -> String {
        let mut key_bytes = [0u8; 32];
        rng().fill_bytes(&mut key_bytes);
        hex::encode(key_bytes)
    }

    fn generate_sha256(raw_token: &str) -> String {
        let mut hasher = Sha256::new();

        hasher.update(raw_token);

        hex::encode(hasher.finalize())
    }

    // TODO: do i really need to pass user id as ref ? check that
    fn generate_access_token(user_id: &Uuid) -> Result<String, VeloxError> {
        let secret = Utils::load_env("JWT_SECRET")?;
        let key = secret.as_bytes();

        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);

        let my_claims = Claims {
            user_id: *user_id,
            exp: expires_at.timestamp() as usize,
        };

        let header = Header::default();

        encode(&header, &my_claims, &EncodingKey::from_secret(key))
            .map_err(|_| VeloxError::InternalError)
    }

    pub fn verify_access_token(token: &str) -> Result<Claims, VeloxError> {
        let secret = Utils::load_env("JWT_SECRET")?;
        let key = secret.as_bytes();

        let result = decode::<Claims>(
            token,
            &DecodingKey::from_secret(key),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| VeloxError::InvalidToken)?;

        Ok(result.claims)
    }

    async fn create_session(
        tx: &mut PgConnection,
        user_id: &Uuid,
    ) -> Result<(String, String), VeloxError> {
        let refresh_token = AuthService::generate_random_token();
        let hashed_refresh_token = AuthService::generate_sha256(&refresh_token);

        let access_token = AuthService::generate_access_token(user_id)?;

        // store hashed_refresh_token inside database
        RefreshTokenRepository::insert(tx, &hashed_refresh_token, &user_id).await?;

        Ok((access_token, refresh_token))
    }

    async fn send_mail(
        response_type: ResponseType,
        user_email: &str,
        raw_token: &str,
    ) -> Result<(), VeloxError> {
        let resend_api_key = Utils::load_env("RESEND_API_KEY")?;

        let resend = resend_rs::Resend::new(&resend_api_key);

        let from = "Velox <onboarding@resend.dev>";
        let to = [user_email];

        let subject = match response_type {
            ResponseType::LogIn => "Your vortex login link",
            ResponseType::SignIn => "Welcome to Velox - verify your email",
        };

        let frontend_url = Utils::load_env("FRONTEND_URL")?;

        let email = CreateEmailBaseOptions::new(from, to, subject).with_html(&format!(
            "<strong>Click <a href='{}/auth/authorize?token={}'>here</a> to verify</strong>",
            frontend_url, raw_token
        ));

        let _email = resend
            .emails
            .send(email)
            .await
            .map_err(|_| VeloxError::EmailError)?;

        Ok(())
    }

    pub async fn login(pool: &Pool<Postgres>, email: &str) -> Result<(), VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        let is_user_exists = UserRepository::find_by_email(&mut tx, email).await?;

        // we will send this in user email
        let token = AuthService::generate_random_token();
        // store this inside database
        let token_sha256 = AuthService::generate_sha256(&token);

        if let Some(_user) = is_user_exists {
            tracing::info!("user already exists proceeding to login logic");
            MagicTokenRepository::delete_by_email(&mut tx, email).await?;
            tracing::info!("deleted old token sucessfully");

            MagicTokenRepository::insert(&mut tx, &token_sha256, email, ResponseType::LogIn)
                .await?;

            tracing::info!("created magic token successfully");

            AuthService::send_mail(ResponseType::LogIn, email, &token).await?;
            tracing::info!("sent email successfully");
        } else {
            tracing::info!("user does not exists");
            MagicTokenRepository::delete_by_email(&mut tx, email).await?;

            TempUserRepository::upsert(&mut tx, email).await?;

            MagicTokenRepository::insert(&mut tx, &token_sha256, email, ResponseType::SignIn)
                .await?;

            AuthService::send_mail(ResponseType::SignIn, email, &token).await?;
            tracing::info!("sent email successfully");
        }

        tx.commit().await.map_err(VeloxError::SqlxError)?;

        Ok(())
    }

    pub async fn authorize(
        pool: &Pool<Postgres>,
        raw_token: &str,
    ) -> Result<(String, String), VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;

        // hash the raw token Sha256
        // we will get token in url like token?=raw_token
        // handler will give us the extracted token
        // query the magic_token table with the token
        // check if exists or not if not send message saying your token is expired
        // or else check the request type if its a login or sign in
        // if login then create the jwt and cookie and login the user
        // ig signin the first promote the temp_user to user table and create jwt and cookies
        // make sure to delete both magic_token and temp_user if type == signin

        let hashed_token = AuthService::generate_sha256(raw_token);

        let magic_token = MagicTokenRepository::find_by_token(&mut tx, &hashed_token)
            .await?
            .ok_or(VeloxError::InvalidToken)?;

        if magic_token.expires_at < chrono::Utc::now() {
            tracing::info!("got expired token");
            return Err(VeloxError::InvalidToken);
        }

        tracing::info!("found valid token");

        let (access_token, refresh_token);

        tracing::debug!("got valid token");

        match magic_token.kind {
            ResponseType::SignIn => {
                tracing::info!("sign in request");
                let temp_user = TempUserRepository::find_by_email(&mut tx, &magic_token.email)
                    .await?
                    .ok_or(VeloxError::InvalidToken)?;

                let user = UserRepository::insert(&mut tx, &temp_user.email).await?;
                TempUserRepository::delete_by_email(&mut tx, &temp_user.email).await?;
                MagicTokenRepository::delete_by_email(&mut tx, &magic_token.email).await?;

                (access_token, refresh_token) =
                    AuthService::create_session(&mut tx, &user.id).await?;

                tracing::info!("created session successfully");
            }
            ResponseType::LogIn => {
                tracing::info!("login request");
                let user = UserRepository::find_by_email(&mut tx, &magic_token.email)
                    .await?
                    .ok_or(VeloxError::NotFound)?;

                MagicTokenRepository::delete_by_email(&mut tx, &magic_token.email).await?;

                (access_token, refresh_token) =
                    AuthService::create_session(&mut tx, &user.id).await?;
                tracing::info!("created session successfully");
            }
        }

        tx.commit().await.map_err(VeloxError::SqlxError)?;

        Ok((access_token, refresh_token))
    }

    pub async fn refresh(
        pool: &Pool<Postgres>,
        raw_refresh_token: &str,
    ) -> Result<(String, String), VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        let hashed_refresh_token = AuthService::generate_sha256(raw_refresh_token);

        // TODO: cycle refresh token delete older one and generate new each time

        let refresh_token_data =
            RefreshTokenRepository::find_by_token(&mut tx, &hashed_refresh_token)
                .await?
                .ok_or(VeloxError::InvalidToken)?;

        if refresh_token_data.expires_at < chrono::Utc::now() {
            tracing::info!("got expired token");
            return Err(VeloxError::InvalidToken);
        }

        let user = UserRepository::find_by_id(&mut tx, &refresh_token_data.user_id)
            .await?
            .ok_or(VeloxError::NotFound)?;

        tracing::info!("found valid user");

        RefreshTokenRepository::delete_by_token(&mut tx, &refresh_token_data.token).await?;

        let refresh_token = AuthService::generate_random_token();
        let hashed_refresh_token = AuthService::generate_sha256(&refresh_token);

        RefreshTokenRepository::insert(&mut tx, &hashed_refresh_token, &user.id).await?;

        let access_token = AuthService::generate_access_token(&user.id)?;

        tx.commit().await.map_err(VeloxError::SqlxError)?;

        Ok((access_token, refresh_token))
    }

    pub async fn logout(pool: &Pool<Postgres>, raw_refresh_token: &str) -> Result<(), VeloxError> {
        let mut tx = pool.begin().await.map_err(VeloxError::SqlxError)?;
        let hashed_refresh_token = AuthService::generate_sha256(raw_refresh_token);

        RefreshTokenRepository::delete_by_token(&mut tx, &hashed_refresh_token).await?;

        tx.commit().await.map_err(VeloxError::SqlxError)?;

        Ok(())
    }
}
