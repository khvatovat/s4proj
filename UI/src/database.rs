use sqlx::{sqlite::SqlitePool, Row};
use std::env;
use dotenv::dotenv;

use crate::models::{User, Credential};

pub async fn establish_connection() -> SqlitePool {
    let database_url = "sqlite://users.db";//env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqlitePool::connect(&database_url).await.unwrap()
}

pub async fn create_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            fingerprint_image BLOB NOT NULL
        )
        "
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn save_user(pool: &SqlitePool, username: &str, fingerprint_image: Vec<u8>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO users (username, fingerprint_image) VALUES (?, ?)
        "
    )
    .bind(username)
    .bind(fingerprint_image)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn save_credentials(pool: &SqlitePool, user_id: i64, site: &str, site_username: &str, site_password: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO credentials (user_id, site, site_username, site_password) VALUES (?, ?, ?, ?)
        "
    )
    .bind(user_id)
    .bind(site)
    .bind(site_username)
    .bind(site_password)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user(pool: &SqlitePool, username: &str) -> Result<Option<User>, sqlx::Error> {
    match sqlx::query("SELECT id, username, fingerprint FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await? {
        Some(row) => Ok(Some(User {
            id: row.get(0),
            username: row.get(1),
            fingerprint_image: row.get(2),
        })),
        None => Ok(None),
    }
}

pub async fn get_credentials(pool: &SqlitePool, user_id: i64) -> Result<Vec<Credential>, sqlx::Error> {
    let rows = sqlx::query("SELECT id, user_id, site, site_username, site_password FROM credentials WHERE user_id = ?")
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let credentials = rows.into_iter().map(|row| Credential {
        id: row.get(0),
        user_id: row.get(1),
        site: row.get(2),
        site_username: row.get(3),
        site_password: row.get(4),
    }).collect();

    Ok(credentials)
}