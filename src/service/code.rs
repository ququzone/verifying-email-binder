use chrono::{DateTime, Utc};
use rand::Rng;
use regex::Regex;
use sqlx::PgPool;

use super::error::ServiceError;
use crate::service::error::Result;

#[derive(Debug, sqlx::FromRow)]
pub struct BindCode {
    pub id: i32,
    pub account: String,
    pub email: String,
    pub code: String,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub async fn generate_code(db: &PgPool, account: String, email: String) -> Result<String> {
    let email_regex = Regex::new(
        r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})",
    )
    .unwrap();
    if !email_regex.is_match(&email) {
        return Err(ServiceError::InvalidRequest(String::from("invalid email")));
    }

    let codes = sqlx::query_as::<_, BindCode>(
        "select id, account, email, code, status, created_at, updated_at from bind_code where account = $1 and email = $2 order by id desc limit 1",
    ).bind(&account).bind(&email).fetch_all(db).await?;

    if !codes.is_empty()
        && codes[0].status < 2
        && codes[0].created_at.timestamp() + 300 > chrono::Local::now().timestamp()
    {
        return Ok("Success".to_string());
    }

    let code = {
        let mut rng = rand::thread_rng();
        rng.gen_range(100000..999999).to_string()
    };

    let _ = sqlx::query(
        r#"INSERT INTO bind_code(account, email, code, status) VALUES ($1, $2, $3, $4)"#,
    )
    .bind(&account)
    .bind(&email)
    .bind(&code)
    .bind(0i16)
    .execute(db)
    .await?;
    Ok("Success".to_string())
}
