//! Database query functions

use sqlx::{Pool, Sqlite, MySql};
use super::{Account, Character, Session};

/// Account queries
pub struct AccountQueries;

impl AccountQueries {
    /// Find account by username
    pub async fn find_by_username(
        pool: &Pool<Sqlite>,
        username: &str,
    ) -> crate::Result<Option<Account>> {
        let account = sqlx::query_as::<_, Account>(
            "SELECT * FROM accounts WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;
        
        Ok(account)
    }
    
    /// Create new account
    pub async fn create(
        pool: &Pool<Sqlite>,
        username: &str,
        password_hash: &str,
    ) -> crate::Result<i64> {
        let result = sqlx::query(
            "INSERT INTO accounts (username, password_hash, created_at, is_banned) VALUES (?, ?, ?, 0)"
        )
        .bind(username)
        .bind(password_hash)
        .bind(chrono::Utc::now().timestamp())
        .execute(pool)
        .await?;
        
        Ok(result.last_insert_rowid())
    }
}

/// Session queries
pub struct SessionQueries;

impl SessionQueries {
    /// Create new session
    pub async fn create(
        pool: &Pool<Sqlite>,
        account_id: i64,
        session_key: &str,
        ttl_seconds: i64,
    ) -> crate::Result<i64> {
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + ttl_seconds;
        
        let result = sqlx::query(
            "INSERT INTO sessions (account_id, session_key, created_at, expires_at, is_active) VALUES (?, ?, ?, ?, 1)"
        )
        .bind(account_id)
        .bind(session_key)
        .bind(now)
        .bind(expires_at)
        .execute(pool)
        .await?;
        
        Ok(result.last_insert_rowid())
    }
    
    /// Validate session key
    pub async fn validate(
        pool: &Pool<Sqlite>,
        session_key: &str,
    ) -> crate::Result<Option<Session>> {
        let now = chrono::Utc::now().timestamp();
        
        let session = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE session_key = ? AND is_active = 1 AND expires_at > ?"
        )
        .bind(session_key)
        .bind(now)
        .fetch_optional(pool)
        .await?;
        
        Ok(session)
    }
}

// Note: Add chrono dependency when implementing these queries
