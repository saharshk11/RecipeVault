use backend::app::{AppState, build_app, default_http_client};
use backend::database;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;


#[tokio::main]
async fn main() {
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

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
