use super::*;

impl From<KqlErrorKind> for KqlError {
    fn from(value: KqlErrorKind) -> Self {
        Self {
            kind: Box::new(value),
        }
    }
}