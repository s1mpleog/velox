use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::storage::init_r2;
use crate::{error::VeloxError, utils::Utils};

pub mod database;
pub mod dto;
pub mod error;
pub mod handlers;
pub mod logger;
pub mod middlewares;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod storage;
pub mod utils;

pub mod app_state;

use crate::database::connect;

#[tokio::main]
async fn main() -> Result<(), VeloxError> {
    dotenvy::dotenv().map_err(|e| VeloxError::EnvError(e.to_string()))?;

    logger::init();

    let database_url = Utils::load_env("DATABASE_URL")?;

    let pool = connect::connect_db(&database_url).await?;

    let r2 = init_r2().await?;

    let app_state = app_state::AppState { pool, r2 };

    let app = Router::new()
        .route("/ping", get(ping))
        .nest("/auth", routes::auth_route::auth_route(app_state.clone()))
        .nest("/folders", routes::folder_route::folder_route())
        .nest("/files", routes::file_route::file_route())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .map_err(|_| VeloxError::InternalError)?;

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .map_err(|_| VeloxError::InternalError)?;

    Ok(())
}

async fn ping() -> Json<serde_json::Value> {
    Json(json!({"msg": "success", "status_code": 200}))
}
