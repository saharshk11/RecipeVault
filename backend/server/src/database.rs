use sqlx::{Row, SqlitePool};

async fn ensure_column(
    pool: &SqlitePool,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), sqlx::Error> {
    let pragma = format!("PRAGMA table_info({table});");
    let rows = sqlx::query(&pragma).fetch_all(pool).await?;
    let mut found = false;
    for row in rows {
        let name: String = row.get("name");
        if name == column {
            found = true;
            break;
        }
    }

    if !found {
        let alter = format!("ALTER TABLE {table} ADD COLUMN {definition};");
        sqlx::query(&alter).execute(pool).await?;
    }

    Ok(())
}

pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Users
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL, -- 'admin' | 'user'
            must_change_password INTEGER NOT NULL DEFAULT 0, -- 0/1
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        "#
    )
    .execute(pool)
    .await?;
    
    // Sessions (DB-backed, cookie points to session id)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        "#
    )
    .execute(pool)
    .await?;

    // Recipes (unique per source URL)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS recipes (
            id TEXT PRIMARY KEY,
            source_url TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            image_url TEXT,
            recipe_json TEXT NOT NULL,
            tags TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Helpful indexes
    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_recipes_title ON recipes(title);"#)
        .execute(pool)
        .await?;
    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);"#)
        .execute(pool)
        .await?;

    ensure_column(
        pool,
        "recipes",
        "notes",
        "notes TEXT NOT NULL DEFAULT '[]'",
    )
    .await?;

    Ok(())

}
