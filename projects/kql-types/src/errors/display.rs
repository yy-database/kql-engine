use super::*;

impl Error for KqlError {}

impl Debug for KqlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.kind, f)
    }
}

impl Display for KqlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.kind, f)
    }
}

impl Display for KqlErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KqlErrorKind::LexicalError { span, message } => {
                write!(f, "Lexical error at {:?}: {}", span, message)
            }
            KqlErrorKind::ParseError { span, message } => {
                write!(f, "Parse error at {:?}: {}", span, message)
            }
            KqlErrorKind::SemanticError { span, message } => {
                write!(f, "Semantic error at {:?}: {}", span, message)
            }
            KqlErrorKind::InternalError { message } => {
                write!(f, "Internal error: {}", message)
            }
        }
    }
}
