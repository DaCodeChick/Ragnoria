//! Database models and queries

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Account model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub created_at: i64,
    pub last_login: Option<i64>,
    pub is_banned: bool,
    pub ban_reason: Option<String>,
}

/// Character model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Character {
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub level: i32,
    pub job_class: i32,
    pub map_id: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
}

/// Session model (for session key management)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: i64,
    pub account_id: i64,
    pub session_key: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub is_active: bool,
}

pub mod queries;
