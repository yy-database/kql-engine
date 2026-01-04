use std::fmt::{Debug, Formatter};
use std::error::Error;
use std::fmt::Display;

mod display;
mod convert;

/// The result type of this crate.
pub type Result<T> = std::result::Result<T, KqlError>;

/// A boxed error kind, wrapping an [KqlErrorKind].
#[derive(Clone)]
pub struct KqlError {
    kind: Box<KqlErrorKind>,
}

/// The kind of [KqlError].
#[derive(Debug, Copy, Clone)]
pub enum KqlErrorKind {
    /// An unknown error.
    UnknownError
}


