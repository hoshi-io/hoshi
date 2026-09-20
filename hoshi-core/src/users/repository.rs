use base64::Engine;
use sqlx::SqlitePool;
use tracing::{debug, instrument};

use crate::error::{CoreError, CoreResult};
use crate::users::types::{UpdateUserBody, UserAuthData, UserModel, UserResponse};

pub struct UserRepo;

impl UserRepo {
    #[instrument(skip(pool))]
    pub async fn get_user_by_id(pool: &SqlitePool, id: i32) -> CoreResult<Option<UserModel>> {
        debug!(id = id, "Fetching user by ID");

        let row: Option<(i32, String, Option<Vec<u8>>, Option<String>, Option<String>)> =
            sqlx::query_as(
                "SELECT id, username, avatar_data, avatar_mime, password_hash FROM User WHERE id = ?",
            )
                .bind(id)
                .fetch_optional(pool)
                .await?;

        Ok(row.map(|(id, username, avatar_data, avatar_mime, password_hash)| {
            let avatar = encode_avatar(avatar_data, avatar_mime);
            UserModel { id, username, avatar, password_hash }
        }))
    }

    pub async fn find_auth_data_by_id(pool: &SqlitePool, user_id: i32) -> CoreResult<Option<UserAuthData>> {
        let row: Option<(String, Option<Vec<u8>>, Option<String>, Option<String>)> =
            sqlx::query_as(
                "SELECT username, avatar_data, avatar_mime, password_hash FROM User WHERE id = ?",
            )
                .bind(user_id)
                .fetch_optional(pool)
                .await?;

        Ok(row.map(|(username, avatar_data, avatar_mime, password_hash)| {
            let avatar = encode_avatar(avatar_data, avatar_mime);
            UserAuthData { username, avatar, password_hash }
        }))
    }

    pub async fn delete_user(pool: &SqlitePool, id: i32) -> CoreResult<bool> {
        let result = sqlx::query("DELETE FROM User WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_all_users(pool: &SqlitePool) -> CoreResult<Vec<UserResponse>> {
        let rows: Vec<(i32, String, Option<Vec<u8>>, Option<String>, i32)> =
            sqlx::query_as(
                r#"
                SELECT id, username, avatar_data, avatar_mime,
                    CASE WHEN password_hash IS NOT NULL THEN 1 ELSE 0 END as has_password
                FROM User ORDER BY id
                "#,
            )
                .fetch_all(pool)
                .await?;

        Ok(rows
            .into_iter()
            .map(|(id, username, avatar_data, avatar_mime, has_password)| {
                let avatar = encode_avatar(avatar_data, avatar_mime);
                UserResponse { id, username, avatar, has_password: has_password == 1 }
            })
            .collect())
    }

    #[instrument(skip(pool))]
    pub async fn create_user(
        pool: &SqlitePool,
        username: &str,
        password_hash: Option<String>,
    ) -> CoreResult<i64> {
        let result = sqlx::query(
            "INSERT INTO User (username, password_hash) VALUES (?, ?)",
        )
            .bind(username)
            .bind(password_hash)
            .execute(pool)
            .await;

        match result {
            Ok(r) => {
                let id = r.last_insert_rowid();
                debug!(id = id, "User record created");
                Ok(id)
            }
            Err(sqlx::Error::Database(e)) if e.message().contains("UNIQUE") => {
                Err(CoreError::Internal("error.user.already_exists".into()))
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn update_user(
        pool: &SqlitePool,
        id: i32,
        updates: &UpdateUserBody,
        new_password_hash: Option<Option<String>>,
    ) -> CoreResult<usize> {
        let mut fields: Vec<&str> = Vec::new();

        if updates.username.is_some() {
            fields.push("username");
        }
        if new_password_hash.is_some() {
            fields.push("password_hash");
        }

        if fields.is_empty() {
            return Ok(0);
        }

        let set_clause = fields.iter()
            .map(|f| format!("{} = ?", f))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!("UPDATE User SET {} WHERE id = ?", set_clause);
        let mut query = sqlx::query(&sql);

        if let Some(username) = &updates.username {
            query = query.bind(username);
        }
        if let Some(ph) = new_password_hash {
            query = query.bind(ph);
        }
        query = query.bind(id);

        let result = query.execute(pool).await?;
        Ok(result.rows_affected() as usize)
    }

    pub async fn get_user_credentials(
        pool: &SqlitePool,
        id: i32,
    ) -> CoreResult<Option<(String, Option<String>)>> {
        let row: Option<(String, Option<String>)> = sqlx::query_as(
            "SELECT username, password_hash FROM User WHERE id = ?",
        )
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(row)
    }

    pub async fn get_password_hash(pool: &SqlitePool, id: i32) -> CoreResult<Option<String>> {
        let row: Option<(Option<String>,)> = sqlx::query_as(
            "SELECT password_hash FROM User WHERE id = ?",
        )
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(row.and_then(|(h,)| h))
    }

    pub async fn update_password(
        pool: &SqlitePool,
        id: i32,
        new_hash: Option<String>,
    ) -> CoreResult<()> {
        sqlx::query("UPDATE User SET password_hash = ? WHERE id = ?")
            .bind(new_hash)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update_avatar(
        pool: &SqlitePool,
        id: i32,
        data: Option<Vec<u8>>,
        mime: Option<String>,
    ) -> CoreResult<()> {
        sqlx::query("UPDATE User SET avatar_data = ?, avatar_mime = ? WHERE id = ?")
            .bind(data)
            .bind(mime)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn get_avatar(
        pool: &SqlitePool,
        id: i32,
    ) -> CoreResult<Option<(Vec<u8>, String)>> {
        let row: Option<(Vec<u8>, String)> = sqlx::query_as(
            "SELECT avatar_data, avatar_mime FROM User WHERE id = ? AND avatar_data IS NOT NULL",
        )
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(row)
    }
}

fn encode_avatar(data: Option<Vec<u8>>, mime: Option<String>) -> Option<String> {
    match (data, mime) {
        (Some(d), Some(m)) => Some(format!(
            "data:{};base64,{}",
            m,
            base64::engine::general_purpose::STANDARD.encode(&d)
        )),
        _ => None,
    }
}