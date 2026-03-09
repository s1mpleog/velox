//
use std::{str::FromStr, time::Duration};

use sqlx::{
    Pool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::error::VeloxError;

pub async fn connect_db(url: &str) -> Result<Pool<Postgres>, VeloxError> {
    let connect_options = PgConnectOptions::from_str(url)
        .map_err(VeloxError::SqlxError)?
        .statement_cache_capacity(0);

    let pool = PgPoolOptions::new()
        .max_connections(10) // TODO: add MAX_CONNECTION in .env and use that
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(connect_options)
        .await
        .map_err(VeloxError::SqlxError)?;

    tracing::info!("connected to database successfully");

    Ok(pool)
}
