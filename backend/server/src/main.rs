use server::app::{AppState, build_app, default_http_client};
use server::database;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let http = default_http_client();

    let options = SqliteConnectOptions::from_str("sqlite:./dev.db")
        .expect("bad sqlite options")
        .create_if_missing(true);

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .expect("failed to connect to database");

    database::init_db(&db).await.expect("failed to init db");
    let state = AppState { http, db };

    let app = build_app(state);

    let addr = std::env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
