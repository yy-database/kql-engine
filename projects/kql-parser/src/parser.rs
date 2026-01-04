use crate::lexer::{Lexer, Token, TokenKind};
use kql_types::{Result, KqlError, Span};
use kql_ast::*;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    prev: Token,
    curr: Token,
    peek: Token,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        
        let mut parser = Self {
            lexer,
            prev: dummy_token(),
            curr: dummy_token(),
            peek: dummy_token(),
        };

        // Initialize tokens, skipping trivia
        parser.advance();
        parser.advance();
        parser.advance();
        parser
    }

    fn advance(&mut self) {
        self.prev = std::mem::replace(&mut self.curr, std::mem::replace(&mut self.peek, self.next_real_token()));
    }

    fn next_real_token(&mut self) -> Token {
        loop {
            let token = self.lexer.next_token();
            match token.kind {
                TokenKind::Whitespace | TokenKind::Comment => continue,
                _ => return token,
            }
        }
    }

    pub fn parse_expression(&mut self, precedence: Precedence) -> Result<Expr> {
        let mut left = self.parse_prefix()?;
        
        while precedence < self.peek_precedence() {
            self.advance();
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr> {
        let token = self.curr.clone();
        
        match token.kind {
            TokenKind::Number => {
                let expr = Expr::Literal(LiteralExpr {
                    kind: LiteralKind::Number(token.text.clone()),
                    span: token.span,
                });
                self.advance();
                Ok(expr)
            }
            TokenKind::Ident => {
                let expr = Expr::Variable(VariableExpr {
                    name: token.text.clone(),
                    span: token.span,
                });
                self.advance();
                Ok(expr)
            }
            TokenKind::Minus => {
                let op_span = token.span.clone();
                self.advance();
                let expr = self.parse_expression(Precedence::Unary)?;
                let span = Span { start: op_span.start, end: expr.span().end };
                Ok(Expr::Unary(UnaryExpr {
                    op: UnaryOp { kind: UnaryOpKind::Neg, span: op_span },
                    expr: Box::new(expr),
                    span,
                }))
            }
            _ => Err(KqlError::ParseError {
                span: token.span,
                message: format!("Unexpected token in prefix position: {:?}", token.kind),
            }),
        }
    }

    fn parse_infix(&mut self, left: Expr) -> Result<Expr> {
        let token = self.curr.clone();
        let precedence = self.curr_precedence();
        
        match token.kind {
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash => {
                let op_kind = match token.kind {
                    TokenKind::Plus => BinaryOpKind::Add,
                    TokenKind::Minus => BinaryOpKind::Sub,
                    TokenKind::Star => BinaryOpKind::Mul,
                    TokenKind::Slash => BinaryOpKind::Div,
                    _ => unreachable!(),
                };
                
                let op = BinaryOp {
                    kind: op_kind,
                    span: token.span.clone(),
                };
                
                self.advance();
                let right = self.parse_expression(precedence)?;
                let span = Span { start: left.span().start, end: right.span().end };
                
                Ok(Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                    span,
                }))
            }
            _ => Ok(left),
        }
    }

    fn curr_precedence(&self) -> Precedence {
        get_precedence(&self.curr.kind)
    }

    fn peek_precedence(&self) -> Precedence {
        get_precedence(&self.peek.kind)
    }
}

fn dummy_token() -> Token {
    Token {
        kind: TokenKind::EOF,
        span: Span { start: 0, end: 0 },
        text: String::new(),
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

fn get_precedence(kind: &TokenKind) -> Precedence {
    match kind {
        TokenKind::Eq => Precedence::Assignment,
        TokenKind::Pipe => Precedence::Or,
        TokenKind::Ampersand => Precedence::And,
        TokenKind::DoubleEq | TokenKind::NotEq => Precedence::Equality,
        TokenKind::Greater | TokenKind::Less | TokenKind::GreaterEq | TokenKind::LessEq => Precedence::Comparison,
        TokenKind::Plus | TokenKind::Minus => Precedence::Sum,
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Product,
        TokenKind::LParen => Precedence::Call,
        TokenKind::Dot => Precedence::Primary,
        _ => Precedence::None,
    }
}
