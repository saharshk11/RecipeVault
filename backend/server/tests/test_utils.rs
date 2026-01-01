use server::app::{AppState, default_http_client};
use server::database;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

#[allow(unused)]
pub async fn test_state() -> AppState {
    let http = default_http_client();
    let db = test_db().await;
    AppState { http, db }
}

pub async fn test_db() -> sqlx::SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .expect("bad sqlite options");

    let db = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("failed to connect to database");

    database::init_db(&db).await.expect("failed to init db");
    db
}
