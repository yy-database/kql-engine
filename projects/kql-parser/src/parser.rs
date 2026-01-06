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

    pub fn parse(&mut self) -> Result<Database> {
        let mut decls = Vec::new();
        let start_pos = self.curr.span.start;
        while !self.is_eof() {
            decls.push(self.parse_declaration()?);
        }
        let end_pos = self.prev.span.end;
        Ok(Database {
            decls,
            span: Span {
                start: start_pos,
                end: end_pos,
            },
        })
    }

    fn parse_attributes(&mut self) -> Result<Vec<Attribute>> {
        let mut attrs = Vec::new();
        while self.curr.kind == TokenKind::At {
            let start_span = self.curr.span.clone();
            self.advance();
            let name = self.parse_ident()?;
            let mut args = None;
            if self.consume(TokenKind::LParen) {
                let mut arg_vec = Vec::new();
                while self.curr.kind != TokenKind::RParen && self.curr.kind != TokenKind::EOF {
                    let mut arg_name = None;
                    // Check if it's a named argument (ident : or keyword :)
                    if (self.curr.kind == TokenKind::Ident || self.is_keyword(&self.curr.kind)) && self.peek.kind == TokenKind::Colon {
                        let token = self.curr.clone();
                        self.advance();
                        arg_name = Some(Ident { name: token.text, span: token.span });
                        self.expect(TokenKind::Colon)?;
                    }

                    let value = self.parse_expression(Precedence::None)?;
                    arg_vec.push(AttributeArg { name: arg_name, value });

                    if !self.consume(TokenKind::Comma) && self.curr.kind != TokenKind::RParen {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                args = Some(arg_vec);
            }
            let end_span = self.prev.span.clone();
            attrs.push(Attribute {
                name,
                args,
                span: Span {
                    start: start_span.start,
                    end: end_span.end,
                },
            });
        }
        Ok(attrs)
    }

    pub fn parse_declaration(&mut self) -> Result<Decl> {
        let attrs = self.parse_attributes()?;
        let decl = match self.curr.kind {
            TokenKind::Struct => self.parse_struct_declaration(attrs).map(Decl::Struct),
            TokenKind::Enum => self.parse_enum_declaration(attrs).map(Decl::Enum),
            TokenKind::Let => self.parse_let_declaration(attrs).map(Decl::Let),
            TokenKind::Namespace => self.parse_namespace_declaration(attrs).map(Decl::Namespace),
            TokenKind::Type => self.parse_type_alias_declaration(attrs).map(Decl::TypeAlias),
            _ => Err(KqlError::parse(
                self.curr.span.clone(),
                format!("Expected declaration (struct, enum, let, namespace, type), found {:?}", self.curr.kind),
            )),
        }?;

        self.consume(TokenKind::Semicolon); // Allow optional semicolon after any declaration
        Ok(decl)
    }

    fn parse_type_alias_declaration(&mut self, attrs: Vec<Attribute>) -> Result<TypeAliasDecl> {
        let start_span = self.expect(TokenKind::Type)?.span;
        let name = self.parse_ident()?;
        self.expect(TokenKind::Eq)?;
        let ty = self.parse_type()?;
        self.consume(TokenKind::Semicolon); // Optional semicolon

        let span = Span {
            start: attrs.first().map(|a| a.span.start).unwrap_or(start_span.start),
            end: ty.span().end,
        };
        Ok(TypeAliasDecl { attrs, name, ty, span })
    }

    fn parse_struct_declaration(&mut self, attrs: Vec<Attribute>) -> Result<StructDecl> {
        let start_span = self.expect(TokenKind::Struct)?.span;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();
        while self.curr.kind != TokenKind::RBrace && self.curr.kind != TokenKind::EOF {
            fields.push(self.parse_field()?);
            self.consume(TokenKind::Comma);
        }

        let end_span = self.expect(TokenKind::RBrace)?.span;
        let span = Span {
            start: attrs.first().map(|a| a.span.start).unwrap_or(start_span.start),
            end: end_span.end,
        };
        Ok(StructDecl { attrs, name, fields, span })
    }

    fn parse_field(&mut self) -> Result<Field> {
        let attrs = self.parse_attributes()?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        let span = Span {
            start: attrs.first().map(|a| a.span.start).unwrap_or(name.span.start),
            end: ty.span().end,
        };
        Ok(Field { attrs, name, ty, span })
    }

    fn parse_enum_declaration(&mut self, attrs: Vec<Attribute>) -> Result<EnumDecl> {
        let start_span = self.expect(TokenKind::Enum)?.span;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut variants = Vec::new();
        while self.curr.kind != TokenKind::RBrace && self.curr.kind != TokenKind::EOF {
            variants.push(self.parse_variant()?);
            self.consume(TokenKind::Comma);
        }

        let end_span = self.expect(TokenKind::RBrace)?.span;
        let span = Span {
            start: attrs.first().map(|a| a.span.start).unwrap_or(start_span.start),
            end: end_span.end,
        };
        Ok(EnumDecl { attrs, name, variants, span })
    }

    fn parse_variant(&mut self) -> Result<Variant> {
        let attrs = self.parse_attributes()?;
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

        let span = Span {
            start: attrs.first().map(|a| a.span.start).unwrap_or(name.span.start),
            end: end_pos,
        };
        Ok(Variant { attrs, name: name.clone(), fields, span })
    }

    fn parse_let_declaration(&mut self, attrs: Vec<Attribute>) -> Result<LetDecl> {
        let start_span = self.expect(TokenKind::Let)?.span;
        let name = self.parse_ident()?;

        let mut ty = None;
        if self.consume(TokenKind::Colon) {
            ty = Some(self.parse_type()?);
        }

        self.expect(TokenKind::Eq)?;
        let value = self.parse_expression(Precedence::None)?;
        let end_span = value.span().end;

        let span = Span {
            start: attrs.first().map(|a| a.span.start).unwrap_or(start_span.start),
            end: end_span,
        };
        Ok(LetDecl { attrs, name, ty, value, span })
    }

    fn parse_namespace_declaration(&mut self, attrs: Vec<Attribute>) -> Result<NamespaceDecl> {
        let start_span = self.expect(TokenKind::Namespace)?.span;
        let name = self.parse_ident()?;

        let mut decls = Vec::new();
        let mut is_block = false;
        let mut end_span = name.span.clone();

        if self.consume(TokenKind::LBrace) {
            is_block = true;
            while self.curr.kind != TokenKind::RBrace && self.curr.kind != TokenKind::EOF {
                decls.push(self.parse_declaration()?);
            }
            end_span = self.expect(TokenKind::RBrace)?.span;
        }

        let span = Span {
            start: attrs.first().map(|a| a.span.start).unwrap_or(start_span.start),
            end: end_span.end,
        };
        Ok(NamespaceDecl {
            attrs,
            name,
            decls,
            is_block,
            span,
        })
    }

    // --- Types ---

    pub fn parse_type(&mut self) -> Result<Type> {
        let token = self.curr.clone();
        let mut ty = match token.kind {
            TokenKind::Ident => {
                let mut name = token.text.clone();
                let start_span = token.span.clone();
                self.advance();

                // Handle qualified names: Namespace::Type
                while self.curr.kind == TokenKind::Colon && self.peek.kind == TokenKind::Colon {
                    self.advance(); // :
                    self.advance(); // :
                    let next_ident = self.parse_ident()?;
                    name.push_str("::");
                    name.push_str(&next_ident.name);
                }

                let mut args = None;
                let mut end_span = self.prev.span.clone();

                if self.curr.kind == TokenKind::Less {
                    self.advance();
                    let mut arg_vec = Vec::new();
                    while self.curr.kind != TokenKind::Greater && self.curr.kind != TokenKind::EOF {
                        let mut arg_name = None;
                        
                        // Check for named argument: name: Type
                        if self.curr.kind == TokenKind::Ident && self.peek.kind == TokenKind::Colon {
                            arg_name = Some(self.parse_ident()?);
                            self.expect(TokenKind::Colon)?;
                        }
                        
                        let arg_ty = self.parse_type()?;
                        arg_vec.push(kql_ast::TypeArg { name: arg_name, ty: arg_ty });

                        if !self.consume(TokenKind::Comma) && self.curr.kind != TokenKind::Greater {
                            break;
                        }
                    }
                    end_span = self.expect(TokenKind::Greater)?.span;
                    args = Some(arg_vec);
                }

                Type::Named(NamedType {
                    name,
                    args,
                    span: Span {
                        start: start_span.start,
                        end: end_span.end,
                    },
                })
            }
            TokenKind::LBracket => {
                let start_span = token.span.clone();
                self.advance();
                let inner = self.parse_type()?;
                let end_span = self.expect(TokenKind::RBracket)?.span;
                Type::List(ListType {
                    inner: Box::new(inner),
                    span: Span {
                        start: start_span.start,
                        end: end_span.end,
                    },
                })
            }
            _ => {
                return Err(KqlError::parse(
                    token.span.clone(),
                    format!("Expected type, found {:?}", token.kind),
                ))
            }
        };

        // Handle Optional (T?)
        while self.curr.kind == TokenKind::Question {
            let q_span = self.curr.span.clone();
            self.advance();
            let start = ty.span().start;
            ty = Type::Optional(OptionalType {
                inner: Box::new(ty),
                span: Span {
                    start,
                    end: q_span.end,
                },
            });
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
                let text = &token.text[1..token.text.len() - 1];
                let expr = Expr::Literal(LiteralExpr { kind: LiteralKind::String(text.to_string()), span: token.span });
                self.advance();
                Ok(expr)
            }
            TokenKind::Boolean => {
                let expr = Expr::Literal(LiteralExpr { kind: LiteralKind::Boolean(token.text == "true"), span: token.span });
                self.advance();
                Ok(expr)
            }
            TokenKind::Null => {
                let expr = Expr::Literal(LiteralExpr { kind: LiteralKind::Null, span: token.span });
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
            TokenKind::Star => {
                let span = self.curr.span.clone();
                self.advance();
                Ok(Expr::Literal(LiteralExpr { kind: LiteralKind::Star, span }))
            }
            TokenKind::LBracket => {
                let start_span = self.curr.span.clone();
                self.advance();
                let mut elements = Vec::new();
                while self.curr.kind != TokenKind::RBracket && self.curr.kind != TokenKind::EOF {
                    elements.push(self.parse_expression(Precedence::None)?);
                    if !self.consume(TokenKind::Comma) && self.curr.kind != TokenKind::RBracket {
                        break;
                    }
                }
                let end_span = self.expect(TokenKind::RBracket)?.span;
                Ok(Expr::List(ListExpr {
                    elements,
                    span: Span { start: start_span.start, end: end_span.end },
                }))
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
                    if self.curr.kind == TokenKind::Ident && self.peek.kind == TokenKind::Colon {
                        let name = self.parse_ident()?;
                        self.expect(TokenKind::Colon)?;
                        let value = self.parse_expression(Precedence::None)?;
                        let span = Span { start: name.span.start, end: value.span().end };
                        args.push(Argument::Named(NamedArgument { name, value, span }));
                    } else {
                        args.push(Argument::Positional(self.parse_expression(Precedence::None)?));
                    }
                    
                    if !self.consume(TokenKind::Comma) && self.curr.kind != TokenKind::RParen {
                        break;
                    }
                }
                let end_span = self.expect(TokenKind::RParen)?.span;
                let span = Span { start: left.span().start, end: end_span.end };
                Ok(Expr::Call(CallExpr { func: Box::new(left), args, span }))
            }
            TokenKind::Dot => {
                self.advance();
                let member = self.parse_ident()?;
                let span = Span { start: left.span().start, end: member.span.end };
                
                if member.name == "over" && self.curr.kind == TokenKind::LParen {
                    let start_pos = left.span().start;
                    self.advance(); // (
                    let mut partition_by = Vec::new();
                    let mut order_by = Vec::new();
                    
                    while self.curr.kind != TokenKind::RParen && self.curr.kind != TokenKind::EOF {
                        let name = self.parse_ident()?;
                        self.expect(TokenKind::Colon)?;
                        
                        match name.name.as_str() {
                            "partition_by" => {
                                let val = self.parse_expression(Precedence::None)?;
                                if let Expr::List(l) = val {
                                    partition_by = l.elements;
                                } else {
                                    partition_by.push(val);
                                }
                            }
                            "order_by" => {
                                let val = self.parse_expression(Precedence::None)?;
                                let elements = if let Expr::List(l) = val {
                                    l.elements
                                } else {
                                    vec![val]
                                };
                                
                                for e in elements {
                                    let mut desc = false;
                                    let mut final_expr = e;
                                    
                                    // Check for .desc()
                                    if let Expr::Call(c) = &final_expr {
                                        if let Expr::Member(m) = &*c.func {
                                            if m.member.name == "desc" {
                                                desc = true;
                                                final_expr = *m.object.clone();
                                            }
                                        }
                                    }
                                    
                                    order_by.push(OrderByExpr {
                                        span: final_expr.span(),
                                        expr: Box::new(final_expr),
                                        desc,
                                    });
                                }
                            }
                            _ => {
                                return Err(KqlError::parse(name.span, format!("Unexpected window argument: {}", name.name)));
                            }
                        }
                        
                        if !self.consume(TokenKind::Comma) && self.curr.kind != TokenKind::RParen {
                            break;
                        }
                    }
                    
                    let end_span = self.expect(TokenKind::RParen)?.span;
                    return Ok(Expr::Window(WindowExpr {
                        expr: Box::new(left),
                        partition_by,
                        order_by,
                        span: Span { start: start_pos, end: end_span.end },
                    }));
                }
                
                Ok(Expr::Member(MemberExpr { object: Box::new(left), member, span }))
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

    fn is_keyword(&self, kind: &TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Struct | TokenKind::Enum | TokenKind::Let | TokenKind::Namespace
        )
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
