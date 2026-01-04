use std::ops::Range;
use thiserror::Error;

/// Source code location span
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

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
