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

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

impl Serialize for CoreError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (key, message) = match self {
            CoreError::Database(e)     => ("error.system.database", e.to_string()),
            CoreError::Io(e)           => ("error.system.io", e.to_string()),
            CoreError::Serialization(e)=> ("error.system.serialization", e.to_string()),
            CoreError::Config(key)     => (key.as_str(), self.to_string()),
            CoreError::NotFound(key)   => (key.as_str(), self.to_string()),
            CoreError::Internal(key)   => (key.as_str(), self.to_string()),
            CoreError::BadRequest(key) => (key.as_str(), self.to_string()),
            CoreError::AuthError(key)  => (key.as_str(), self.to_string()),
            CoreError::Network(key)    => (key.as_str(), self.to_string()),
            CoreError::Parse(key)      => (key.as_str(), self.to_string()),
            CoreError::Validation(key) => (key.as_str(), self.to_string()),
        };

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