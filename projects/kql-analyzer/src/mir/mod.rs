use std::collections::HashMap;

pub mod mir_gen;

#[derive(Debug, Clone, PartialEq)]
pub struct MirDatabase {
    pub tables: HashMap<String, Table>,
}

impl Default for MirDatabase {
    fn default() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub primary_key: Option<Vec<String>>,
    pub indexes: Vec<Index>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub name: String,
    pub ty: ColumnType,
    pub nullable: bool,
    pub auto_increment: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}
