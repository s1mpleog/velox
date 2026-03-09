use rand::{Rng, rng};
use resend_rs::types::CreateEmailBaseOptions;
use sha2::{Digest, Sha256};
use sqlx::{Pool, Postgres};

use crate::error::VeloxError;
use crate::models::magic_token_model::ResponseType;
use crate::repositories::magic_token_repository::MagicTokenRepository;
use crate::repositories::temp_user_repository::TempUserRepository;
use crate::repositories::user_repository::UserRepository;
use crate::utils::Utils;

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

        // TODO: use DEPLOYMENT_URL instead of hardcoding localhost
        let email = CreateEmailBaseOptions::new(from, to, subject)
        .with_html(&format!("<strong>Click <a href='http://localhost:3000/auth/authorize?token={}'>here</a> to verify</strong>", raw_token));

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

        if let Some(user) = is_user_exists {
            tracing::info!("User exists with email {}", user.email);

            MagicTokenRepository::delete_by_email(&mut tx, email).await?;

            MagicTokenRepository::insert(&mut tx, &token_sha256, email, ResponseType::LogIn)
                .await?;

            AuthService::send_mail(ResponseType::LogIn, email, &token).await?;
        } else {
            tracing::info!("User does not exists sign in");

            MagicTokenRepository::delete_by_email(&mut tx, email).await?;

            TempUserRepository::upsert(&mut tx, email).await?;

            tracing::info!("Generated random token: {}", token);

            tracing::info!("Sha 256 token: {}", token_sha256);

            MagicTokenRepository::insert(&mut tx, &token_sha256, email, ResponseType::SignIn)
                .await?;

            AuthService::send_mail(ResponseType::SignIn, email, &token).await?;
        }

        tx.commit().await.map_err(VeloxError::SqlxError)?;

        Ok(())
    }

    pub async fn authorize(pool: &Pool<Postgres>, raw_token: &str) {}
}
