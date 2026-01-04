use super::*;

impl From<KqlErrorKind> for KqlError {
    fn from(value: KqlErrorKind) -> Self {
        Self { kind: Box::new(value) }
    }
}

impl From<std::io::Error> for KqlError {
    fn from(error: std::io::Error) -> Self {
        Self::io(error.to_string())
    }
}

#[cfg(feature = "serde")]
impl From<serde_json::Error> for KqlError {
    fn from(error: serde_json::Error) -> Self {
        Self::internal(format!("JSON error: {}", error))
    }
}

#[cfg(feature = "sqlx")]
impl From<sqlx::Error> for KqlError {
    fn from(error: sqlx::Error) -> Self {
        Self::database(error.to_string())
    }
}
