use crate::lexer::{Lexer, Token, TokenKind};
use kql_types::{Result, KqlError};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    prev: Token,
    curr: Token,
    peek: Token,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let curr = lexer.next_token();
        let peek = lexer.next_token();
        
        // Initial dummy token for prev
        let prev = Token {
            kind: TokenKind::EOF,
            span: kql_types::Span { start: 0, end: 0 },
            text: String::new(),
        };

        Self { lexer, prev, curr, peek }
    }

    fn advance(&mut self) {
        self.prev = std::mem::replace(&mut self.curr, std::mem::replace(&mut self.peek, self.lexer.next_token()));
    }

    // Pratt parsing entry point for expressions
    pub fn parse_expression(&mut self, precedence: Precedence) -> Result<()> {
        // TODO: Implement Pratt parsing logic
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Sum,
    Product,
    Unary,
    Call,
    Primary,
}
