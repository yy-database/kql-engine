use kql_types::Span;
use std::str::Chars;
use std::iter::Peekable;

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
    Type,
    
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
    DoubleArrow, // =>
    Dollar,      // $
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    Pipe,        // |
    Ampersand,   // &
    
    // Literals
    Ident,
    String,
    Number,
    Boolean,
    
    // Trivia (Important for Lossless parsing)
    Whitespace,
    Comment,
    
    // Special
    Error(String),
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
    chars: Peekable<Chars<'a>>,
    cursor: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            cursor: 0,
        }
    }

    pub fn next_token(&mut self) -> Token {
        let start = self.cursor;
        
        let kind = match self.advance() {
            Some(c) => match c {
                // Whitespace
                c if c.is_whitespace() => {
                    self.consume_while(|c| c.is_whitespace());
                    TokenKind::Whitespace
                }
                
                // Comments
                '/' if self.peek() == Some('/') => {
                    self.consume_while(|c| c != '\n');
                    TokenKind::Comment
                }
                '/' if self.peek() == Some('*') => {
                    self.advance(); // consume '*'
                    while let Some(c) = self.advance() {
                        if c == '*' && self.peek() == Some('/') {
                            self.advance(); // consume '/'
                            break;
                        }
                    }
                    TokenKind::Comment
                }
                
                // Numbers
                c if c.is_ascii_digit() => {
                    self.consume_while(|c| c.is_ascii_digit() || c == '.');
                    TokenKind::Number
                }
                
                // Identifiers & Keywords
                c if c.is_alphabetic() || c == '_' => {
                    self.consume_while(|c| c.is_alphanumeric() || c == '_');
                    let text = &self.source[start..self.cursor];
                    match text {
                        "struct" => TokenKind::Struct,
                        "enum" => TokenKind::Enum,
                        "let" => TokenKind::Let,
                        "for" => TokenKind::For,
                        "in" => TokenKind::In,
                        "yield" => TokenKind::Yield,
                        "if" => TokenKind::If,
                        "else" => TokenKind::Else,
                        "match" => TokenKind::Match,
                        "case" => TokenKind::Case,
                        "lambda" => TokenKind::Lambda,
                        "type" => TokenKind::Type,
                        "true" => TokenKind::Boolean,
                        "false" => TokenKind::Boolean,
                        _ => TokenKind::Ident,
                    }
                }
                
                // Strings
                '"' => {
                    while let Some(c) = self.peek() {
                        if c == '"' {
                            self.advance();
                            break;
                        }
                        if c == '\\' {
                            self.advance();
                        }
                        self.advance();
                    }
                    TokenKind::String
                }

                // Symbols
                '{' => TokenKind::LBrace,
                '}' => TokenKind::RBrace,
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                '[' => TokenKind::LBracket,
                ']' => TokenKind::RBracket,
                ':' => TokenKind::Colon,
                ';' => TokenKind::Semicolon,
                ',' => TokenKind::Comma,
                '.' => TokenKind::Dot,
                '?' => TokenKind::Question,
                '@' => TokenKind::At,
                '$' => TokenKind::Dollar,
                '+' => TokenKind::Plus,
                '-' => if self.peek() == Some('>') { self.advance(); TokenKind::Arrow } else { TokenKind::Minus },
                '*' => TokenKind::Star,
                '/' => TokenKind::Slash,
                '%' => TokenKind::Percent,
                '|' => TokenKind::Pipe,
                '&' => TokenKind::Ampersand,
                '=' => match self.peek() {
                    Some('=') => { self.advance(); TokenKind::DoubleEq }
                    Some('>') => { self.advance(); TokenKind::DoubleArrow }
                    _ => TokenKind::Eq,
                },
                '!' => if self.peek() == Some('=') { self.advance(); TokenKind::NotEq } else { TokenKind::Error("Unexpected !".into()) },
                '>' => if self.peek() == Some('=') { self.advance(); TokenKind::GreaterEq } else { TokenKind::Greater },
                '<' => if self.peek() == Some('=') { self.advance(); TokenKind::LessEq } else { TokenKind::Less },
                
                _ => TokenKind::Error(format!("Unexpected character: {}", c)),
            }
            None => TokenKind::EOF,
        };

        let end = self.cursor;
        Token {
            kind,
            span: Span { start, end },
            text: self.source[start..end].to_string(),
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next();
        if let Some(c) = c {
            self.cursor += c.len_utf8();
        }
        c
    }

    fn consume_while<F>(&mut self, mut f: F)
    where
        F: FnMut(char) -> bool,
    {
        while let Some(c) = self.peek() {
            if f(c) {
                self.advance();
            } else {
                break;
            }
        }
    }
}
