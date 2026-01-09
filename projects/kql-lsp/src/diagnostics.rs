//! Diagnostic utilities for KQL LSP

use kql_types::{KqlError, KqlErrorKind, Span};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// Converts a KQL error to an LSP diagnostic
pub fn error_to_diagnostic(error: &KqlError, text: &str) -> Diagnostic {
    let span = match error.kind() {
        KqlErrorKind::LexicalError { span, .. } => *span,
        KqlErrorKind::ParseError { span, .. } => *span,
        KqlErrorKind::SemanticError { span, .. } => *span,
        KqlErrorKind::LintError { span, .. } => *span,
        _ => Span { start: 0, end: 10 },
    };

    Diagnostic {
        range: span_to_range(text, span),
        severity: Some(DiagnosticSeverity::ERROR),
        code: None,
        code_description: None,
        source: Some("kql-lsp".to_string()),
        message: error.to_string(),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Convert KQL span to LSP range
pub fn span_to_range(text: &str, span: Span) -> Range {
    let start = position_from_offset(text, span.start);
    let end = position_from_offset(text, span.end);
    
    Range { start, end }
}

/// Convert byte offset to LSP position
fn position_from_offset(text: &str, offset: usize) -> Position {
    if offset > text.len() {
        return Position { line: 0, character: 0 };
    }
    
    let slice = &text[..offset];
    let lines: Vec<&str> = slice.lines().collect();
    let line = lines.len().saturating_sub(1) as u32;
    let character = lines.last().map(|l| l.len() as u32).unwrap_or(0);

    Position { line, character }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_to_range() {
        let text = "struct User {\n    id: i32\n}";
        let span = Span { start: 7, end: 11 }; // "User"
        let range = span_to_range(text, span);
        
        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 7);
        assert_eq!(range.end.line, 0);
        assert_eq!(range.end.character, 11);
    }

    #[test]
    fn test_span_to_range_multiline() {
        let text = "struct User {\n    id: i32\n}";
        let span = Span { start: 19, end: 22 }; // "i32"
        let range = span_to_range(text, span);
        
        assert_eq!(range.start.line, 1);
        assert_eq!(range.end.line, 1);
    }
}
