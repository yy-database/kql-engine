use indexmap::IndexMap;
use serde::{Serialize, Deserialize};

pub mod mir_gen;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirProgram {
    pub tables: IndexMap<String, Table>,
}

impl Default for MirProgram {
    fn default() -> Self {
        Self {
            tables: IndexMap::new(),
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
    pub foreign_key_column: String,
    pub target_table: String,
    pub target_column: String,
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
