use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use chrono::Utc;
use rand::distr::Alphanumeric;
use rand::Rng;
use rand_core::OsRng;
use secrecy::{ExposeSecret, SecretString};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;
use std::collections::HashMap;

use crate::error::ApiError;

pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_USER: &str = "user";

pub const DEFAULT_SESSION_TTL_SECS: i64 = 60 * 60 * 24 * 30;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub role: String,
    pub must_change_password: bool,
    pub tag_colors: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone)]
pub struct GeneratedCredentials {
    pub username: String,
    pub password: SecretString,
}

#[derive(Debug, Clone)]
pub struct AdminBootstrap {
    pub user: AuthUser,
    pub generated_credentials: Option<GeneratedCredentials>,
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn bool_from_sqlite(value: i64) -> bool {
    value != 0
}

fn parse_tag_colors(value: Option<String>) -> HashMap<String, String> {
    match value {
        Some(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        None => HashMap::new(),
    }
}

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(ApiError::password_hash)?
        .to_string();

    Ok(hash)
}

pub fn verify_password(password_hash: &str, password: &str) -> Result<bool, ApiError> {
    let parsed = PasswordHash::new(password_hash).map_err(ApiError::password_hash)?;
    let argon2 = Argon2::default();
    let result = argon2.verify_password(password.as_bytes(), &parsed);
    Ok(result.is_ok())
}

pub fn generate_random_credentials() -> GeneratedCredentials {
    let mut rng = rand::rng();
    let username = format!(
        "admin_{}",
        (&mut rng)
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect::<String>()
    );
    let password = (&mut rng)
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect::<String>();

    GeneratedCredentials {
        username,
        password: SecretString::new(password.into_boxed_str()),
    }
}

pub async fn find_user_by_id(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Option<AuthUser>, ApiError> {
    let row_opt = sqlx::query(
        r#"
        SELECT id, username, role, must_change_password, tag_colors
        FROM users
        WHERE id = ?1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::db_read)?;

    Ok(row_opt.map(|row| AuthUser {
        id: row.get("id"),
        username: row.get("username"),
        role: row.get("role"),
        must_change_password: bool_from_sqlite(row.get::<i64, _>("must_change_password")),
        tag_colors: parse_tag_colors(row.get("tag_colors")),
    }))
}

pub async fn find_user_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<AuthUser>, ApiError> {
    let row_opt = sqlx::query(
        r#"
        SELECT id, username, role, must_change_password, tag_colors
        FROM users
        WHERE username = ?1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::db_read)?;

    Ok(row_opt.map(|row| AuthUser {
        id: row.get("id"),
        username: row.get("username"),
        role: row.get("role"),
        must_change_password: bool_from_sqlite(row.get::<i64, _>("must_change_password")),
        tag_colors: parse_tag_colors(row.get("tag_colors")),
    }))
}

pub async fn verify_credentials(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<Option<AuthUser>, ApiError> {
    let row_opt = sqlx::query(
        r#"
        SELECT id, username, password_hash, role, must_change_password, tag_colors
        FROM users
        WHERE username = ?1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::db_read)?;

    let row = match row_opt {
        Some(row) => row,
        None => return Ok(None),
    };

    let password_hash: String = row.get("password_hash");
    if !verify_password(&password_hash, password)? {
        return Ok(None);
    }

    Ok(Some(AuthUser {
        id: row.get("id"),
        username: row.get("username"),
        role: row.get("role"),
        must_change_password: bool_from_sqlite(row.get::<i64, _>("must_change_password")),
        tag_colors: parse_tag_colors(row.get("tag_colors")),
    }))
}

pub async fn create_user(
    pool: &SqlitePool,
    username: &str,
    password: &str,
    role: &str,
    must_change_password: bool,
) -> Result<AuthUser, ApiError> {
    let password_hash = hash_password(password)?;
    let id = Uuid::new_v4().to_string();
    let now = now_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO users (id, username, password_hash, role, must_change_password, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )
    .bind(&id)
    .bind(username)
    .bind(&password_hash)
    .bind(role)
    .bind(if must_change_password { 1 } else { 0 })
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ApiError::db_write)?;

    Ok(AuthUser {
        id,
        username: username.to_string(),
        role: role.to_string(),
        must_change_password,
        tag_colors: HashMap::new(),
    })
}

pub async fn update_user_credentials(
    pool: &SqlitePool,
    user_id: &str,
    new_username: &str,
    new_password: &str,
    role: &str,
    must_change_password: bool,
) -> Result<AuthUser, ApiError> {
    let password_hash = hash_password(new_password)?;
    let now = now_rfc3339();

    sqlx::query(
        r#"
        UPDATE users
        SET username = ?1,
            password_hash = ?2,
            must_change_password = ?3,
            updated_at = ?4
        WHERE id = ?5
        "#,
    )
    .bind(new_username)
    .bind(&password_hash)
    .bind(if must_change_password { 1 } else { 0 })
    .bind(&now)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(ApiError::db_write)?;

    let row = sqlx::query(
        r#"
        SELECT tag_colors
        FROM users
        WHERE id = ?1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(ApiError::db_read)?;

    Ok(AuthUser {
        id: user_id.to_string(),
        username: new_username.to_string(),
        role: role.to_string(),
        must_change_password,
        tag_colors: parse_tag_colors(row.get("tag_colors")),
    })
}

pub async fn verify_user_password(
    pool: &SqlitePool,
    user_id: &str,
    password: &str,
) -> Result<bool, ApiError> {
    let row_opt = sqlx::query(
        r#"
        SELECT password_hash
        FROM users
        WHERE id = ?1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::db_read)?;

    let row = match row_opt {
        Some(row) => row,
        None => return Ok(false),
    };

    let password_hash: String = row.get("password_hash");
    verify_password(&password_hash, password)
}

pub async fn create_session(
    pool: &SqlitePool,
    user_id: &str,
    ttl_secs: i64,
) -> Result<Session, ApiError> {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now();
    let expires_at = created_at + chrono::Duration::seconds(ttl_secs);

    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, created_at, expires_at)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(&id)
    .bind(user_id)
    .bind(created_at.to_rfc3339())
    .bind(expires_at.to_rfc3339())
    .execute(pool)
    .await
    .map_err(ApiError::db_write)?;

    Ok(Session {
        id,
        user_id: user_id.to_string(),
        created_at: created_at.to_rfc3339(),
        expires_at: expires_at.to_rfc3339(),
    })
}

pub async fn delete_session(pool: &SqlitePool, session_id: &str) -> Result<(), ApiError> {
    sqlx::query(
        r#"
        DELETE FROM sessions
        WHERE id = ?1
        "#,
    )
    .bind(session_id)
    .execute(pool)
    .await
    .map_err(ApiError::db_write)?;

    Ok(())
}

pub async fn find_user_by_session_id(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<AuthUser>, ApiError> {
    let now = now_rfc3339();

    let row_opt = sqlx::query(
        r#"
        SELECT u.id, u.username, u.role, u.must_change_password, u.tag_colors
        FROM sessions s
        JOIN users u ON u.id = s.user_id
        WHERE s.id = ?1 AND s.expires_at > ?2
        "#,
    )
    .bind(session_id)
    .bind(now)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::db_read)?;

    Ok(row_opt.map(|row| AuthUser {
        id: row.get("id"),
        username: row.get("username"),
        role: row.get("role"),
        must_change_password: bool_from_sqlite(row.get::<i64, _>("must_change_password")),
        tag_colors: parse_tag_colors(row.get("tag_colors")),
    }))
}

pub async fn update_user_tag_colors(
    pool: &SqlitePool,
    user_id: &str,
    tag_colors: &HashMap<String, String>,
) -> Result<AuthUser, ApiError> {
    let now = now_rfc3339();
    let json = serde_json::to_string(tag_colors).map_err(ApiError::serialization)?;

    sqlx::query(
        r#"
        UPDATE users
        SET tag_colors = ?1,
            updated_at = ?2
        WHERE id = ?3
        "#,
    )
    .bind(&json)
    .bind(&now)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(ApiError::db_write)?;

    let row = sqlx::query(
        r#"
        SELECT id, username, role, must_change_password, tag_colors
        FROM users
        WHERE id = ?1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(ApiError::db_read)?;

    Ok(AuthUser {
        id: row.get("id"),
        username: row.get("username"),
        role: row.get("role"),
        must_change_password: bool_from_sqlite(row.get::<i64, _>("must_change_password")),
        tag_colors: parse_tag_colors(row.get("tag_colors")),
    })
}

pub async fn cleanup_expired_sessions(pool: &SqlitePool) -> Result<u64, ApiError> {
    let now = now_rfc3339();
    let result = sqlx::query(
        r#"
        DELETE FROM sessions
        WHERE expires_at <= ?1
        "#,
    )
    .bind(now)
    .execute(pool)
    .await
    .map_err(ApiError::db_write)?;

    Ok(result.rows_affected())
}

pub async fn ensure_admin_user(
    pool: &SqlitePool,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<AdminBootstrap, ApiError> {
    let existing_admin = sqlx::query(
        r#"
        SELECT id, username, role, must_change_password, tag_colors
        FROM users
        WHERE role = ?1
        LIMIT 1
        "#,
    )
    .bind(ROLE_ADMIN)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::db_read)?;

    if let Some(row) = existing_admin {
        return Ok(AdminBootstrap {
            user: AuthUser {
                id: row.get("id"),
                username: row.get("username"),
                role: row.get("role"),
                must_change_password: bool_from_sqlite(row.get::<i64, _>("must_change_password")),
                tag_colors: parse_tag_colors(row.get("tag_colors")),
            },
            generated_credentials: None,
        });
    }

    let (final_username, final_password, generated) = match (username, password) {
        (Some(user), Some(pass)) => (
            user.to_string(),
            SecretString::new(pass.to_owned().into_boxed_str()),
            None,
        ),
        _ => {
            let creds = generate_random_credentials();
            (
                creds.username.clone(),
                creds.password.clone(),
                Some(creds),
            )
        }
    };

    let must_change_password = generated.is_some();
    let user = create_user(
        pool,
        &final_username,
        final_password.expose_secret(),
        ROLE_ADMIN,
        must_change_password,
    )
    .await?;

    Ok(AdminBootstrap {
        user,
        generated_credentials: generated,
    })
}
