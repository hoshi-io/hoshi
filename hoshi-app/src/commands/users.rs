use crate::{require_auth, TauriSession};
use hoshi_core::users::types::UserResponse;
use hoshi_core::users::types::{ChangePasswordBody, DeleteUserBody, UpdateUserBody, UserPrivate};
use hoshi_core::{
    error::CoreError,
    state::AppState,
    users::service::UserService
};
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

#[derive(Serialize)]
pub struct UsersListResponse {
    pub users: Vec<UserResponse>,
}

#[tauri::command]
pub async fn get_all_users(
    state: State<'_, Arc<AppState>>
) -> Result<UsersListResponse, CoreError> {
    let users = UserService::get_all_users(&state).await?;
    Ok(UsersListResponse { users })
}

#[tauri::command]
pub async fn get_me(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<UserPrivate, CoreError> {
    let user_id = require_auth(&session_state).await?;
    UserService::get_me(&state, user_id).await
}

#[tauri::command]
pub async fn update_me(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    updates: UpdateUserBody,
) -> Result<(), CoreError> {
    let user_id = require_auth(&session_state).await?;
    UserService::update_user(&state, user_id, updates).await
}

#[tauri::command]
pub async fn delete_me(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: DeleteUserBody,
) -> Result<(), CoreError> {
    let user_id = require_auth(&session_state).await?;
    UserService::delete_user(&state, user_id, body).await
}

#[tauri::command]
pub async fn change_password(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: ChangePasswordBody,
) -> Result<bool, CoreError> {
    let user_id = require_auth(&session_state).await?;
    UserService::change_password(&state, user_id, body).await
}

#[tauri::command]
pub async fn upload_avatar(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    data: Vec<u8>,
    content_type: String,
) -> Result<(), CoreError> {
    let user_id = require_auth(&session_state).await?;
    UserService::upload_avatar(&state, user_id, data, content_type).await
}

#[tauri::command]
pub async fn delete_avatar(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<(), CoreError> {
    let user_id = require_auth(&session_state).await?;
    UserService::delete_avatar(&state, user_id).await
}