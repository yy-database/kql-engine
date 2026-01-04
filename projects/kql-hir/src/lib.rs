use kql_types::Span;
use std::collections::HashMap;

pub mod lower;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HirType {
    Primitive(PrimitiveType),
    Struct(HirId),
    Enum(HirId),
    List(Box<HirType>),
    Optional(Box<HirType>),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PrimitiveType {
    Int,
    Float,
    String,
    Bool,
    DateTime,
    Uuid,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirExpr {
    pub kind: HirExprKind,
    pub ty: HirType,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HirExprKind {
    Literal(HirLiteral),
    Binary { left: Box<HirExpr>, op: HirBinaryOp, right: Box<HirExpr> },
    Unary { op: HirUnaryOp, expr: Box<HirExpr> },
    Variable(HirId),
    Call { func: Box<HirExpr>, args: Vec<HirExpr> },
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HirLiteral {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HirBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Gt,
    Lt,
    GtEq,
    LtEq,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HirUnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirStruct {
    pub id: HirId,
    pub name: String,
    pub fields: Vec<HirField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirField {
    pub name: String,
    pub ty: HirType,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirEnum {
    pub id: HirId,
    pub name: String,
    pub variants: Vec<HirVariant>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirVariant {
    pub name: String,
    pub fields: Option<Vec<HirField>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirLet {
    pub id: HirId,
    pub name: String,
    pub ty: HirType,
    pub value: HirExpr,
    pub span: Span,
}

#[derive(Debug, Default)]
pub struct HirDatabase {
    pub structs: HashMap<HirId, HirStruct>,
    pub enums: HashMap<HirId, HirEnum>,
    pub lets: HashMap<HirId, HirLet>,
    pub name_to_id: HashMap<String, HirId>,
    next_id: usize,
}

impl HirDatabase {
    pub fn alloc_id(&mut self) -> HirId {
        let id = HirId(self.next_id);
        self.next_id += 1;
        id
    }
}
