use bcrypt::{hash, verify, DEFAULT_COST};
use tracing::{error, info, instrument, warn};

use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use crate::users::repository::UserRepo;
use crate::users::types::{
    ChangePasswordBody, DeleteUserBody, UpdateUserBody,
    UserPrivate, UserResponse,
};

pub struct UserService;

impl UserService {
    pub async fn get_all_users(state: &AppState) -> CoreResult<Vec<UserResponse>> {
        UserRepo::get_all_users(state.pool()).await
    }

    #[instrument(skip(state))]
    pub async fn get_me(state: &AppState, user_id: i32) -> CoreResult<UserPrivate> {
        let user = UserRepo::get_user_by_id(state.pool(), user_id)
            .await?
            .ok_or_else(|| {
                warn!("User not found in session");
                CoreError::NotFound("error.user.not_found".into())
            })?;

        Ok(UserPrivate {
            id: user.id,
            username: user.username,
            avatar: user.avatar,
            has_password: user.password_hash.is_some(),
        })
    }

    #[instrument(skip(state, updates), fields(id = id))]
    pub async fn update_user(
        state: &AppState,
        id: i32,
        updates: UpdateUserBody,
    ) -> CoreResult<()> {
        if updates.username.is_none() && updates.password.is_none() {
            return Err(CoreError::BadRequest("error.user.no_updates_provided".into()));
        }

        let password_hash_update = if let Some(password) = &updates.password {
            if password.is_empty() {
                Some(None)
            } else {
                let h = hash(password.trim(), DEFAULT_COST).map_err(|e| {
                    error!(error = ?e, "Failed to hash new password during update");
                    CoreError::Internal("error.user.hashing_failed".into())
                })?;
                Some(Some(h))
            }
        } else {
            None
        };

        let changes = UserRepo::update_user(state.pool(), id, &updates, password_hash_update).await?;

        if changes > 0 {
            info!("User profile updated successfully");
            Ok(())
        } else {
            Err(CoreError::NotFound("error.user.not_found".into()))
        }
    }

    #[instrument(skip(state, body), fields(id = id))]
    pub async fn delete_user(state: &AppState, id: i32, body: DeleteUserBody) -> CoreResult<()> {
        let (_, password_hash) = UserRepo::get_user_credentials(state.pool(), id)
            .await?
            .ok_or_else(|| CoreError::NotFound("error.user.not_found".into()))?;

        if let Some(hash_str) = password_hash {
            let pass = body.password.ok_or_else(|| {
                warn!("Account deletion rejected: password required");
                CoreError::AuthError("error.user.password_required".into())
            })?;

            if !verify(&pass, &hash_str).unwrap_or(false) {
                warn!("Account deletion rejected: incorrect password");
                return Err(CoreError::AuthError("error.user.incorrect_password".into()));
            }
        }

        let success = UserRepo::delete_user(state.pool(), id).await?;
        if success {
            info!("User account deleted successfully");
            Ok(())
        } else {
            error!("Failed to delete user record from database");
            Err(CoreError::Internal("error.user.delete_failed".into()))
        }
    }

    #[instrument(skip(state, body), fields(id = id))]
    pub async fn change_password(
        state: &AppState,
        id: i32,
        body: ChangePasswordBody,
    ) -> CoreResult<bool> {
        let password_hash = UserRepo::get_password_hash(state.pool(), id).await?;

        if let Some(hash_str) = password_hash {
            let curr_pass = body.current_password.as_ref().ok_or_else(|| {
                CoreError::AuthError("error.user.current_password_required".into())
            })?;

            if !verify(curr_pass, &hash_str).unwrap_or(false) {
                warn!("Password change rejected: current password incorrect");
                return Err(CoreError::AuthError("error.user.current_password_incorrect".into()));
            }
        }

        let has_new_password = body.new_password.is_some();
        let new_hash = match &body.new_password {
            Some(pass) if !pass.is_empty() => {
                Some(hash(pass.trim(), DEFAULT_COST).map_err(|e| {
                    error!(error = ?e, "Failed to hash new password during change");
                    CoreError::Internal("error.user.hashing_failed".into())
                })?)
            }
            _ => None,
        };

        UserRepo::update_password(state.pool(), id, new_hash).await?;
        info!("User password changed successfully");
        Ok(has_new_password)
    }

    #[instrument(skip(state, data), fields(id = id, mime = %mime))]
    pub async fn upload_avatar(
        state: &AppState,
        id: i32,
        data: Vec<u8>,
        mime: String,
    ) -> CoreResult<()> {
        match mime.as_str() {
            "image/jpeg" | "image/png" | "image/webp" | "image/gif" => {}
            _ => return Err(CoreError::BadRequest("error.user.unsupported_image_format".into())),
        }

        if data.len() > 20 * 1024 * 1024 {
            warn!("Avatar upload rejected: file size {} exceeds 20MB", data.len());
            return Err(CoreError::BadRequest("error.user.image_too_large".into()));
        }

        UserRepo::update_avatar(state.pool(), id, Some(data), Some(mime)).await?;
        info!("User avatar updated successfully");
        Ok(())
    }


    #[instrument(skip(state))]
    pub async fn delete_avatar(state: &AppState, id: i32) -> CoreResult<()> {
        UserRepo::update_avatar(state.pool(), id, None, None).await?;
        info!(user_id = id, "User avatar deleted successfully");
        Ok(())
    }
}