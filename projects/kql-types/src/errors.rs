use thiserror::Error;
use crate::Span;

/// KQL Error definitions
#[derive(Debug, Error)]
pub enum KqlError {
    #[error("Lexer error at {span:?}: {message}")]
    LexicalError {
        span: Span,
        message: String,
    },

    #[error("Parser error at {span:?}: {message}")]
    ParseError {
        span: Span,
        message: String,
    },

    #[error("Semantic error at {span:?}: {message}")]
    SemanticError {
        span: Span,
        message: String,
    },

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, KqlError>;
