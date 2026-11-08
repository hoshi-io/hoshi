use tracing::{info, instrument, warn};

use crate::auth::repository::AuthRepository;
use crate::auth::types::{LoginRequest, RegisterRequest, UserInfo};
use crate::core_err;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use crate::users::repository::UserRepo;
use crate::users::password::{hash_password, verify_password};

pub struct AuthService;

impl AuthService {
    #[instrument(skip(state, payload), fields(user_id = payload.user_id))]
    pub async fn login(state: &AppState, payload: LoginRequest) -> CoreResult<UserInfo> {
        if payload.user_id <= 0 {
            warn!("Login rejected: invalid user ID");
            return Err(CoreError::BadRequest("error.auth.invalid_user_id".into()));
        }

        let auth_data = UserRepo::find_auth_data_by_id(&state.pool, payload.user_id)
            .await?
            .ok_or_else(|| {
                warn!("Login rejected: user not found");
                CoreError::NotFound("error.auth.user_not_found".into())
            })?;

        let has_password = auth_data.password_hash.is_some();

        if let Some(hash_str) = auth_data.password_hash {
            let password_input = payload.password.ok_or_else(|| {
                warn!("Login rejected: password required but not provided");
                CoreError::AuthError("error.auth.password_required".into())
            })?;

            let is_valid = verify_password(password_input, hash_str).await
                .map_err(|e| core_err!(Internal, "error.auth.password_verification_failed", e))?;

            if !is_valid {
                warn!("Login rejected: incorrect password");
                return Err(CoreError::AuthError("error.auth.incorrect_password".into()));
            }
        }

        AuthRepository::set_active_user(&state.pool, Some(payload.user_id)).await?;

        info!("User logged in successfully");
        Ok(UserInfo {
            id:       payload.user_id,
            username: auth_data.username,
            avatar:   auth_data.avatar,
            has_password,
        })
    }

    #[instrument(skip(state, payload), fields(username = %payload.username))]
    pub async fn register(state: &AppState, payload: RegisterRequest) -> CoreResult<UserInfo> {
        if payload.username.is_empty() {
            warn!("Registration rejected: empty username");
            return Err(CoreError::BadRequest("error.auth.username_required".into()));
        }

        let password_hash = match payload.password.as_deref().map(str::trim).filter(|p| !p.is_empty()) {
            Some(p) => Some(hash_password(p.to_string()).await
                .map_err(|e| core_err!(Internal, "error.auth.hashing_failed", e))?),
            None => None,
        };

        let user_id = UserRepo::create_user(&state.pool, &payload.username, password_hash).await?;

        info!(user_id = user_id, "New user registered");
        AuthRepository::set_active_user(&state.pool, Some(user_id as i32)).await?;

        Ok(UserInfo {
            id: user_id as i32,
            username: payload.username,
            avatar: None,
            has_password: payload.password.as_deref().map(str::trim).filter(|p| !p.is_empty()).is_some(),
        })
    }

    #[instrument(skip(state))]
    pub async fn logout(state: &AppState) -> CoreResult<()> {
        info!("User logged out");
        AuthRepository::set_active_user(&state.pool, None).await
    }

    pub async fn get_active_user(state: &AppState) -> CoreResult<Option<UserInfo>> {
        let Some(user_id) = AuthRepository::get_active_user(&state.pool).await? else {
            return Ok(None);
        };

        let Some(auth_data) = UserRepo::find_auth_data_by_id(&state.pool, user_id).await? else {
            return Ok(None);
        };

        Ok(Some(UserInfo {
            id: user_id,
            username: auth_data.username,
            avatar: auth_data.avatar,
            has_password: auth_data.password_hash.is_some(),
        }))
    }
}