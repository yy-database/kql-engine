use crate::Span;
use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

mod convert;
mod display;

/// The result type of this crate.
pub type Result<T> = std::result::Result<T, KqlError>;

/// A boxed error kind, wrapping an [KqlErrorKind].
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KqlError {
    kind: Box<KqlErrorKind>,
}

impl KqlError {
    pub fn new(kind: KqlErrorKind) -> Self {
        Self { kind: Box::new(kind) }
    }

    pub fn kind(&self) -> &KqlErrorKind {
        &self.kind
    }

    pub fn lexical(span: Span, message: impl Into<String>) -> Self {
        Self::new(KqlErrorKind::LexicalError { span, message: message.into() })
    }

    pub fn parse(span: Span, message: impl Into<String>) -> Self {
        Self::new(KqlErrorKind::ParseError { span, message: message.into() })
    }

    pub fn semantic(span: Span, message: impl Into<String>) -> Self {
        Self::new(KqlErrorKind::SemanticError { span, message: message.into() })
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(KqlErrorKind::IoError { message: message.into() })
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::new(KqlErrorKind::DatabaseError { message: message.into() })
    }

    pub fn cli(message: impl Into<String>) -> Self {
        Self::new(KqlErrorKind::CliError { message: message.into() })
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(KqlErrorKind::InternalError { message: message.into() })
    }
}

/// The kind of [KqlError].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KqlErrorKind {
    LexicalError { span: Span, message: String },
    ParseError { span: Span, message: String },
    SemanticError { span: Span, message: String },
    IoError { message: String },
    DatabaseError { message: String },
    CliError { message: String },
    InternalError { message: String },
}
