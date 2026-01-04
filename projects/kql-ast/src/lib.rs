use kql_types::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trivia {
    Whitespace(String),
    Comment(String),
}

pub trait AstNode {
    fn span(&self) -> Span;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Literal(LiteralExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Variable(VariableExpr),
    Call(CallExpr),
}

impl AstNode for Expr {
    fn span(&self) -> Span {
        match self {
            Expr::Literal(e) => e.span,
            Expr::Binary(e) => e.span,
            Expr::Unary(e) => e.span,
            Expr::Variable(e) => e.span,
            Expr::Call(e) => e.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralExpr {
    pub kind: LiteralKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralKind {
    Number(String),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: BinaryOp,
    pub right: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryOp {
    pub kind: BinaryOpKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOpKind {
    Add, Sub, Mul, Div, Mod,
    Eq, NotEq, Gt, Lt, GtEq, LtEq,
    And, Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnaryOp {
    pub kind: UnaryOpKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOpKind {
    Neg, Not,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableExpr {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallExpr {
    pub func: Box<Expr>,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decl {
    Struct(StructDecl),
    Enum(EnumDecl),
    Let(LetDecl),
}

impl AstNode for Decl {
    fn span(&self) -> Span {
        match self {
            Decl::Struct(d) => d.span,
            Decl::Enum(d) => d.span,
            Decl::Let(d) => d.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDecl {
    pub name: Ident,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDecl {
    pub name: Ident,
    pub variants: Vec<Variant>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetDecl {
    pub name: Ident,
    pub ty: Option<Type>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    pub name: Ident,
    pub fields: Option<Vec<Field>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Named(NamedType),
    List(ListType),
    Optional(OptionalType),
}

impl AstNode for Type {
    fn span(&self) -> Span {
        match self {
            Type::Named(t) => t.span,
            Type::List(t) => t.span,
            Type::Optional(t) => t.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedType {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListType {
    pub inner: Box<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionalType {
    pub inner: Box<Type>,
    pub span: Span,
}
