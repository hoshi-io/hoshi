use thiserror::Error;
use serde::Serialize;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Auth error: {0}")]
    AuthError(String),

    #[error("Network error: {0}")]
    Network(String),

    /// The remote asked us to slow down (HTTP 429).
    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// Looks like a translation key: `error.<area>.<name>` (lowercase, digits, `_`).
fn is_key(s: &str) -> bool {
    s.starts_with("error.")
        && !s.ends_with('.')
        && !s.contains("..")
        && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'.')
}

impl CoreError {
    /// Key used when the payload isn't a translation key (raw `e.to_string()`, free text).
    fn generic_key(&self) -> &'static str {
        match self {
            CoreError::Database(_)      => "error.system.database",
            CoreError::Io(_)            => "error.system.io",
            CoreError::Serialization(_) => "error.system.serialization",
            CoreError::Config(_)        => "error.config.invalid",
            CoreError::NotFound(_)      => "error.system.not_found",
            CoreError::Internal(_)      => "error.system.internal",
            CoreError::BadRequest(_)    => "error.system.bad_request",
            CoreError::AuthError(_)     => "error.auth.failed",
            CoreError::Network(_)       => "error.system.network",
            CoreError::RateLimited(_)   => "error.system.rate_limited",
            CoreError::Parse(_)         => "error.system.parse",
            CoreError::Validation(_)    => "error.system.validation",
        }
    }

    fn payload(&self) -> Option<&str> {
        match self {
            CoreError::Config(s) | CoreError::NotFound(s) | CoreError::Internal(s)
            | CoreError::BadRequest(s) | CoreError::AuthError(s) | CoreError::Network(s)
            | CoreError::RateLimited(s) | CoreError::Parse(s) | CoreError::Validation(s) => Some(s),
            CoreError::Database(_) | CoreError::Io(_) | CoreError::Serialization(_) => None,
        }
    }

    /// `(key, message)` sent to the frontend. The payload may be a key, `"key: detail"`,
    /// or plain text (falls back to the variant's generic key, text kept as the message).
    pub fn key_and_message(&self) -> (&str, String) {
        match self.payload() {
            Some(p) if is_key(p) => (p, self.to_string()),
            Some(p) => match p.split_once(": ") {
                Some((k, detail)) if is_key(k) => (k, detail.to_string()),
                _ => (self.generic_key(), p.to_string()),
            },
            None => (self.generic_key(), self.to_string()),
        }
    }
}

impl Serialize for CoreError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (key, message) = self.key_and_message();

        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("CoreError", 2)?;
        state.serialize_field("key", key)?;
        state.serialize_field("message", &message)?;
        state.end()
    }
}

pub type CoreResult<T> = Result<T, CoreError>;

#[macro_export]
macro_rules! core_err {
    ($variant:ident, $key:expr) => {{
        tracing::error!($key);
        $crate::error::CoreError::$variant($key.into())
    }};

    ($variant:ident, $key:expr, $err:expr) => {{
        let __e = &$err;
        tracing::error!(error = ?__e, $key);
        $crate::error::CoreError::$variant($key.into())
    }};

    ($variant:ident, $key:expr, $err:expr, $($fields:tt)+) => {{
        let __e = &$err;
        tracing::error!(error = ?__e, $($fields)+, $key);
        $crate::error::CoreError::$variant($key.into())
    }};
}
