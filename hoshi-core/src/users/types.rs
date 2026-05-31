use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPrivate {
    pub id: i32,
    pub username: String,
    pub avatar: Option<String>,
    pub has_password: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserBody {
    pub username: String,
    pub password: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserBody {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub avatar: Option<String>,
    pub has_password: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordBody {
    pub current_password: Option<String>,
    pub new_password: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteUserBody {
    pub password: Option<String>,
}

pub struct UserAuthData {
    pub username: String,
    pub avatar: Option<String>,
    pub password_hash: Option<String>,
}

pub struct UserModel {
    pub id: i32,
    pub username: String,
    pub avatar: Option<String>,
    pub password_hash: Option<String>,
}