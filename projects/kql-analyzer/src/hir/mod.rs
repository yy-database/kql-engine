use kql_types::Span;
use indexmap::IndexMap;

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
    Key {
        entity: Option<HirId>,
        inner: Box<HirType>,
    },
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PrimitiveType {
    I32,
    I64,
    F32,
    F64,
    String,
    Bool,
    DateTime,
    Uuid,
    D128,
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
    Symbol(String),
    Call { func: Box<HirExpr>, args: Vec<HirExpr> },
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HirLiteral {
    Integer64(i64),
    Float64(f64),
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

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirAttribute {
    pub name: String,
    pub args: Vec<HirAttributeArg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirAttributeArg {
    pub name: Option<String>,
    pub value: HirExpr,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirStruct {
    pub id: HirId,
    pub attrs: Vec<HirAttribute>,
    pub name: String,
    pub namespace: Option<String>,
    pub schema: Option<String>,
    pub fields: Vec<HirField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirField {
    pub attrs: Vec<HirAttribute>,
    pub name: String,
    pub ty: HirType,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirEnum {
    pub id: HirId,
    pub attrs: Vec<HirAttribute>,
    pub name: String,
    pub namespace: Option<String>,
    pub schema: Option<String>,
    pub variants: Vec<HirVariant>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirVariant {
    pub attrs: Vec<HirAttribute>,
    pub name: String,
    pub fields: Option<Vec<HirField>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirLet {
    pub id: HirId,
    pub attrs: Vec<HirAttribute>,
    pub name: String,
    pub namespace: Option<String>,
    pub ty: HirType,
    pub value: HirExpr,
    pub span: Span,
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirProgram {
    pub structs: IndexMap<HirId, HirStruct>,
    pub enums: IndexMap<HirId, HirEnum>,
    pub lets: IndexMap<HirId, HirLet>,
    pub name_to_id: IndexMap<String, HirId>,
    pub id_to_kind: IndexMap<HirId, HirKind>,
    next_id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HirKind {
    Struct,
    Enum,
    Let,
}

impl HirProgram {
    pub fn alloc_id(&mut self) -> HirId {
        let id = HirId(self.next_id);
        self.next_id += 1;
        id
    }
}
