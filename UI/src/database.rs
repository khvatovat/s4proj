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
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            fingerprint_image BLOB NOT NULL
        )
        "#
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS credentials (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            site TEXT NOT NULL,
            site_username TEXT NOT NULL,
            site_password TEXT NOT NULL
        )
        "#
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

pub async fn save_credentials(pool: &SqlitePool, username: &str, site: &str, site_username: &str, site_password: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO credentials (username, site, site_username, site_password) VALUES (?, ?, ?, ?)
        "
    )
    .bind(username)
    .bind(site)
    .bind(site_username)
    .bind(site_password)
    .fetch_all(pool)
    .await?;

    Ok(())
}

pub async fn get_user(pool: &SqlitePool, username: &str) -> Result<Option<User>, sqlx::Error> {
    match sqlx::query("SELECT id, username, fingerprint_image FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await? {
        Some(row) => Ok(Some(User {
            id: row.get("id"),
            username: row.get("username"),
            fingerprint_image: row.get("fingerprint_image"),
        })),
        None => Ok(None),
    }
}

pub async fn get_credentials(pool: &SqlitePool, username: &str) -> Result<Vec<Credential>, sqlx::Error> {
    let rows = sqlx::query("SELECT username, site, site_username, site_password FROM credentials WHERE username = ?")
    .bind(username)
    .fetch_all(pool)
    .await?;

    let credentials = rows.into_iter().map(|row| Credential {
        id: 0,
        username: row.get("username"),
        site: row.get("site"),
        site_username: row.get("site_username"),
        site_password: row.get("site_password"),
    }).collect();

    Ok(credentials)
}