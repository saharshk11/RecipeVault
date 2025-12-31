use recipe_core::extract_recipe;
use secrecy::ExposeSecret;
use url::Url;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

fn normalize_sqlite_url(input: &str) -> String {
    if input.starts_with("sqlite:") {
        input.to_string()
    } else {
        format!("sqlite:{input}")
    }
}

fn print_usage() {
    eprintln!("usage:");
    eprintln!("  cli <url>");
    eprintln!("  cli admin-bootstrap [--db <path-or-url>] [--username <u>] [--password <p>]");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let Some(first) = args.next() else {
        print_usage();
        anyhow::bail!("missing arguments");
    };

    if first == "admin-bootstrap" {
        run_admin_bootstrap(args).await?;
        return Ok(());
    }

    let url = Url::parse(&first)?;

    let html = reqwest::get(url.clone()).await?.text().await?;
    let recipe = extract_recipe(&html, &url)?;

    println!("{}", serde_json::to_string_pretty(&recipe)?);
    Ok(())
}

async fn run_admin_bootstrap(
    mut args: impl Iterator<Item = String>,
) -> anyhow::Result<()> {
    let mut db = None;
    let mut username = None;
    let mut password = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--db" => db = args.next(),
            "--username" => username = args.next(),
            "--password" => password = args.next(),
            _ => {
                print_usage();
                anyhow::bail!("unknown argument: {arg}");
            }
        }
    }

    let db_url = normalize_sqlite_url(db.as_deref().unwrap_or("./dev.db"));
    let options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    backend::database::init_db(&pool).await?;

    let bootstrap = backend::auth::ensure_admin_user(
        &pool,
        username.as_deref(),
        password.as_deref(),
    )
    .await
    .map_err(|e| anyhow::anyhow!(e.message))?;

    println!("admin user: {}", bootstrap.user.username);
    if let Some(generated) = bootstrap.generated_credentials {
        println!("generated username: {}", generated.username);
        println!("generated password: {}", generated.password.expose_secret());
        println!("must_change_password: true");
    } else {
        println!("must_change_password: {}", bootstrap.user.must_change_password);
    }

    Ok(())
}
