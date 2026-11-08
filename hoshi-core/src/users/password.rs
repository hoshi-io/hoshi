use crate::core_err;
use crate::error::CoreResult;

pub async fn hash_password(password: String) -> CoreResult<String> {
    tokio::task::spawn_blocking(move || bcrypt::hash(password.trim(), bcrypt::DEFAULT_COST))
        .await
        .map_err(|e| core_err!(Internal, "error.auth.hashing_failed", e))?
        .map_err(|e| core_err!(Internal, "error.auth.hashing_failed", e))
}

pub async fn verify_password(password: String, hash: String) -> CoreResult<bool> {
    tokio::task::spawn_blocking(move || bcrypt::verify(password.trim(), &hash))
        .await
        .map_err(|e| core_err!(Internal, "error.auth.password_verification_failed", e))?
        .map_err(|e| core_err!(Internal, "error.auth.password_verification_failed", e))
}
