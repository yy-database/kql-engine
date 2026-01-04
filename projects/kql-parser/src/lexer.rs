use kql_types::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Struct,
    Enum,
    Let,
    For,
    In,
    Yield,
    If,
    Else,
    Match,
    Case,
    Lambda,
    
    // Symbols
    LBrace,      // {
    RBrace,      // }
    LParen,      // (
    RParen,      // )
    LBracket,    // [
    RBracket,    // ]
    Colon,       // :
    Semicolon,   // ;
    Comma,       // ,
    Dot,         // .
    Question,    // ?
    At,          // @
    Eq,          // =
    DoubleEq,    // ==
    NotEq,       // !=
    Greater,     // >
    Less,        // <
    GreaterEq,   // >=
    LessEq,      // <=
    Arrow,       // ->
    Dollar,      // $
    
    // Literals
    Ident,
    String(String),
    Number(String),
    Boolean(bool),
    
    // Trivia
    Whitespace,
    Comment,
    
    // Special
    EOF,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

pub struct Lexer<'a> {
    source: &'a str,
    cursor: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, cursor: 0 }
    }

    // TODO: Implement next_token with Pratt Parser logic and comment preservation
    pub fn next_token(&mut self) -> Token {
        // Placeholder
        Token {
            kind: TokenKind::EOF,
            span: Span { start: self.cursor, end: self.cursor },
            text: String::new(),
        }
    }
}
