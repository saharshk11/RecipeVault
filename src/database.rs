use sqlx::SqlitePool;

pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Users
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL
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

    // User <-> Recipe saves
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_recipes (
            user_id TEXT NOT NULL,
            recipe_id TEXT NOT NULL,
            saved_at TEXT NOT NULL,
            PRIMARY KEY (user_id, recipe_id),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Helpful indexes
    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_recipes_title ON recipes(title);"#)
        .execute(pool)
        .await?;

    Ok(())

}