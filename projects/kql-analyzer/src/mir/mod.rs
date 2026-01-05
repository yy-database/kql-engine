use indexmap::IndexMap;
use serde::{Serialize, Deserialize};

pub mod mir_gen;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirProgram {
    pub tables: IndexMap<String, Table>,
    pub queries: IndexMap<String, MirQuery>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirQuery {
    pub name: String,
    pub source_table: String,
    pub joins: Vec<MirJoin>,
    pub selection: Option<MirExpr>,
    pub projection: Vec<MirProjection>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirJoin {
    pub relation_name: String, // The name of the relation field in KQL
    pub target_table: String,
    pub join_type: MirJoinType,
    pub condition: Option<MirExpr>, // Custom condition if any
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MirJoinType {
    Inner,
    Left,
    Right,
    Full,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MirProjection {
    All,
    Field(String),
    Alias(String, Box<MirExpr>),
    Aggregation(MirAggregation),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirAggregation {
    pub func: String,
    pub arg: Box<MirExpr>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MirExpr {
    Column { table_alias: Option<String>, column: String },
    Literal(MirLiteral),
    Binary { left: Box<MirExpr>, op: MirBinaryOp, right: Box<MirExpr> },
    Unary { op: MirUnaryOp, expr: Box<MirExpr> },
    Call { func: String, args: Vec<MirExpr> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MirLiteral {
    Integer64(i64),
    Float64(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MirBinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, NotEq, Gt, Lt, GtEq, LtEq,
    And, Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MirUnaryOp {
    Neg, Not,
}

impl Default for MirProgram {
    fn default() -> Self {
        Self {
            tables: IndexMap::new(),
            queries: IndexMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub schema: Option<String>,
    pub name: String,
    pub columns: Vec<Column>,
    pub primary_key: Option<Vec<String>>,
    pub indexes: Vec<Index>,
    pub foreign_keys: Vec<ForeignKey>,
    pub relations: Vec<Relation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relation {
    pub name: String,
    pub relation_name: Option<String>, // name from @relation(name: "...")
    pub foreign_key_column: String,
    pub target_table: String,
    pub target_column: String,
    pub is_list: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForeignKey {
    pub name: String,
    pub columns: Vec<String>,
    pub referenced_schema: Option<String>,
    pub referenced_table: String,
    pub referenced_columns: Vec<String>,
    pub on_delete: Option<ReferenceAction>,
    pub on_update: Option<ReferenceAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceAction {
    NoAction,
    Restrict,
    Cascade,
    SetNull,
    SetDefault,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub ty: ColumnType,
    pub nullable: bool,
    pub auto_increment: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColumnType {
    I16,
    I32,
    I64,
    F32,
    F64,
    String(Option<usize>), // Optional length
    Bool,
    DateTime,
    Uuid,
    Json,
    Decimal128,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}
