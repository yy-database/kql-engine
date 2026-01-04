use crate::lexer::{Lexer, Token, TokenKind};
use kql_ast::*;
use kql_types::{KqlError, Result, Span};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    prev: Token,
    curr: Token,
    peek: Token,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let lexer = Lexer::new(source);

        let mut parser = Self { lexer, prev: dummy_token(), curr: dummy_token(), peek: dummy_token() };

        // Initialize tokens, skipping trivia
        parser.advance();
        parser.advance();
        parser
    }

    pub fn is_eof(&self) -> bool {
        self.curr.kind == TokenKind::EOF
    }

    fn advance(&mut self) {
        let next = self.next_real_token();
        self.prev = std::mem::replace(&mut self.curr, std::mem::replace(&mut self.peek, next));
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

    fn expect(&mut self, kind: TokenKind) -> Result<Token> {
        if self.curr.kind == kind {
            let token = self.curr.clone();
            self.advance();
            Ok(token)
        }
        else {
            Err(KqlError::parse(self.curr.span.clone(), format!("Expected {:?}, found {:?}", kind, self.curr.kind)))
        }
    }

    fn consume(&mut self, kind: TokenKind) -> bool {
        if self.curr.kind == kind {
            self.advance();
            true
        }
        else {
            false
        }
    }

    // --- Declarations ---

    pub fn parse_declaration(&mut self) -> Result<Decl> {
        match self.curr.kind {
            TokenKind::Struct => self.parse_struct_declaration().map(Decl::Struct),
            TokenKind::Enum => self.parse_enum_declaration().map(Decl::Enum),
            TokenKind::Let => self.parse_let_declaration().map(Decl::Let),
            _ => Err(KqlError::parse(
                self.curr.span.clone(),
                format!("Expected declaration (struct, enum, let), found {:?}", self.curr.kind),
            )),
        }
    }

    fn parse_struct_declaration(&mut self) -> Result<StructDecl> {
        let start_span = self.expect(TokenKind::Struct)?.span;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();
        while self.curr.kind != TokenKind::RBrace && self.curr.kind != TokenKind::EOF {
            fields.push(self.parse_field()?);
            if !self.consume(TokenKind::Comma) && self.curr.kind != TokenKind::RBrace {
                break;
            }
        }

        let end_span = self.expect(TokenKind::RBrace)?.span;
        Ok(StructDecl { name, fields, span: Span { start: start_span.start, end: end_span.end } })
    }

    fn parse_field(&mut self) -> Result<Field> {
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        let span = Span { start: name.span.start, end: ty.span().end };
        Ok(Field { name, ty, span })
    }

    fn parse_enum_declaration(&mut self) -> Result<EnumDecl> {
        let start_span = self.expect(TokenKind::Enum)?.span;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut variants = Vec::new();
        while self.curr.kind != TokenKind::RBrace && self.curr.kind != TokenKind::EOF {
            variants.push(self.parse_variant()?);
            if !self.consume(TokenKind::Comma) && self.curr.kind != TokenKind::RBrace {
                break;
            }
        }

        let end_span = self.expect(TokenKind::RBrace)?.span;
        Ok(EnumDecl { name, variants, span: Span { start: start_span.start, end: end_span.end } })
    }

    fn parse_variant(&mut self) -> Result<Variant> {
        let name = self.parse_ident()?;
        let mut fields = None;
        let mut end_pos = name.span.end;

        if self.consume(TokenKind::LBrace) {
            let mut f_vec = Vec::new();
            while self.curr.kind != TokenKind::RBrace && self.curr.kind != TokenKind::EOF {
                f_vec.push(self.parse_field()?);
                if !self.consume(TokenKind::Comma) && self.curr.kind != TokenKind::RBrace {
                    break;
                }
            }
            end_pos = self.expect(TokenKind::RBrace)?.span.end;
            fields = Some(f_vec);
        }

        Ok(Variant { name: name.clone(), fields, span: Span { start: name.span.start, end: end_pos } })
    }

    fn parse_let_declaration(&mut self) -> Result<LetDecl> {
        let start_span = self.expect(TokenKind::Let)?.span;
        let name = self.parse_ident()?;

        let mut ty = None;
        if self.consume(TokenKind::Colon) {
            ty = Some(self.parse_type()?);
        }

        self.expect(TokenKind::Eq)?;
        let value = self.parse_expression(Precedence::None)?;
        let end_span = value.span().end;

        Ok(LetDecl { name, ty, value, span: Span { start: start_span.start, end: end_span } })
    }

    // --- Types ---

    pub fn parse_type(&mut self) -> Result<Type> {
        let token = self.curr.clone();
        let mut ty = match token.kind {
            TokenKind::Ident => {
                let name = token.text.clone();
                let span = token.span.clone();
                self.advance();
                Type::Named(NamedType { name, span })
            }
            TokenKind::LBracket => {
                let start_span = token.span.clone();
                self.advance();
                let inner = self.parse_type()?;
                let end_span = self.expect(TokenKind::RBracket)?.span;
                Type::List(ListType { inner: Box::new(inner), span: Span { start: start_span.start, end: end_span.end } })
            }
            _ => return Err(KqlError::parse(token.span.clone(), format!("Expected type, found {:?}", token.kind))),
        };

        // Handle Optional (T?)
        if self.curr.kind == TokenKind::Question {
            let q_span = self.curr.span.clone();
            self.advance();
            let start = ty.span().start;
            ty = Type::Optional(OptionalType { inner: Box::new(ty), span: Span { start, end: q_span.end } });
        }

        Ok(ty)
    }

    fn parse_ident(&mut self) -> Result<Ident> {
        let token = self.expect(TokenKind::Ident)?;
        Ok(Ident { name: token.text, span: token.span })
    }

    // --- Expressions ---

    pub fn parse_expression(&mut self, precedence: Precedence) -> Result<Expr> {
        let mut left = self.parse_prefix()?;

        while precedence < self.curr_precedence() {
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr> {
        let token = self.curr.clone();

        match token.kind {
            TokenKind::Number => {
                let expr = Expr::Literal(LiteralExpr { kind: LiteralKind::Number(token.text.clone()), span: token.span });
                self.advance();
                Ok(expr)
            }
            TokenKind::String => {
                let expr = Expr::Literal(LiteralExpr { kind: LiteralKind::String(token.text.clone()), span: token.span });
                self.advance();
                Ok(expr)
            }
            TokenKind::Boolean => {
                let expr = Expr::Literal(LiteralExpr { kind: LiteralKind::Boolean(token.text == "true"), span: token.span });
                self.advance();
                Ok(expr)
            }
            TokenKind::Ident => {
                let expr = Expr::Variable(VariableExpr { name: token.text.clone(), span: token.span });
                self.advance();
                Ok(expr)
            }
            TokenKind::Minus => {
                let op_span = token.span.clone();
                self.advance();
                let expr = self.parse_expression(Precedence::Unary)?;
                let span = Span { start: op_span.start, end: expr.span().end };
                Ok(Expr::Unary(UnaryExpr { op: UnaryOp { kind: UnaryOpKind::Neg, span: op_span }, expr: Box::new(expr), span }))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression(Precedence::None)?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            _ => Err(KqlError::parse(token.span, format!("Unexpected token in prefix position: {:?}", token.kind))),
        }
    }

    fn parse_infix(&mut self, left: Expr) -> Result<Expr> {
        let token = self.curr.clone();
        let precedence = self.curr_precedence();

        match token.kind {
            TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::DoubleEq
            | TokenKind::NotEq
            | TokenKind::Greater
            | TokenKind::Less
            | TokenKind::GreaterEq
            | TokenKind::LessEq
            | TokenKind::Ampersand
            | TokenKind::Pipe => {
                let op_kind = match token.kind {
                    TokenKind::Plus => BinaryOpKind::Add,
                    TokenKind::Minus => BinaryOpKind::Sub,
                    TokenKind::Star => BinaryOpKind::Mul,
                    TokenKind::Slash => BinaryOpKind::Div,
                    TokenKind::Percent => BinaryOpKind::Mod,
                    TokenKind::DoubleEq => BinaryOpKind::Eq,
                    TokenKind::NotEq => BinaryOpKind::NotEq,
                    TokenKind::Greater => BinaryOpKind::Gt,
                    TokenKind::Less => BinaryOpKind::Lt,
                    TokenKind::GreaterEq => BinaryOpKind::GtEq,
                    TokenKind::LessEq => BinaryOpKind::LtEq,
                    TokenKind::Ampersand => BinaryOpKind::And,
                    TokenKind::Pipe => BinaryOpKind::Or,
                    _ => unreachable!(),
                };

                let op = BinaryOp { kind: op_kind, span: token.span.clone() };

                self.advance();
                let right = self.parse_expression(precedence)?;
                let span = Span { start: left.span().start, end: right.span().end };

                Ok(Expr::Binary(BinaryExpr { left: Box::new(left), op, right: Box::new(right), span }))
            }
            TokenKind::LParen => {
                self.advance();
                let mut args = Vec::new();
                while self.curr.kind != TokenKind::RParen && self.curr.kind != TokenKind::EOF {
                    args.push(self.parse_expression(Precedence::None)?);
                    if !self.consume(TokenKind::Comma) && self.curr.kind != TokenKind::RParen {
                        break;
                    }
                }
                let end_span = self.expect(TokenKind::RParen)?.span;
                let span = Span { start: left.span().start, end: end_span.end };
                Ok(Expr::Call(CallExpr { func: Box::new(left), args, span }))
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
    Token { kind: TokenKind::EOF, span: Span { start: 0, end: 0 }, text: String::new() }
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
