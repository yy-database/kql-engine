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
        match self { KqlErrorKind::UnknownError => { write!(f, "UnknownError") } }
    }
}