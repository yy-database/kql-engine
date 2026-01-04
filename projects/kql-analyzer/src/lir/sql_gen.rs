use crate::mir::{Column, ColumnType, MirDatabase, Table};
use sqlparser::ast::{
    CharacterLength, ColumnDef, DataType, Ident, ObjectName, Statement, TableConstraint,
};

pub struct SqlGenerator {
    pub mir_db: MirDatabase,
}

impl SqlGenerator {
    pub fn new(mir_db: MirDatabase) -> Self {
        Self { mir_db }
    }

    pub fn generate_ddl(&self) -> Vec<Statement> {
        let mut statements = Vec::new();
        for table in self.mir_db.tables.values() {
            statements.push(self.generate_create_table(table));
        }
        statements
    }

    fn generate_create_table(&self, table: &Table) -> Statement {
        let mut columns = Vec::new();
        for col in &table.columns {
            columns.push(self.generate_column_def(col));
        }

        let mut constraints = Vec::new();
        if let Some(pk_cols) = &table.primary_key {
            constraints.push(TableConstraint::Unique {
                name: None,
                columns: pk_cols.iter().map(|c| Ident::new(c)).collect(),
                is_primary: true,
                characteristics: None,
            });
        }

        Statement::CreateTable {
            or_replace: false,
            temporary: false,
            external: false,
            if_not_exists: true,
            transient: false,
            name: ObjectName(vec![Ident::new(&table.name)]),
            columns,
            constraints,
            with_options: vec![],
            file_format: None,
            location: None,
            query: None,
            without_rowid: false,
            like: None,
            clone: None,
            engine: None,
            comment: None,
            default_charset: None,
            collation: None,
            on_commit: None,
            on_cluster: None,
            order_by: None,
            partition_by: None,
            cluster_by: None,
            options: None,
            strict: false,
            global: None,
            hive_distribution: sqlparser::ast::HiveDistributionStyle::NONE,
            hive_formats: None,
            table_properties: vec![],
            auto_increment_offset: None,
        }
    }

    fn generate_column_def(&self, col: &Column) -> ColumnDef {
        let data_type = match &col.ty {
            ColumnType::Int32 => DataType::Int(None),
            ColumnType::Int64 => DataType::BigInt(None),
            ColumnType::Float32 => DataType::Float(None),
            ColumnType::Float64 => DataType::Double,
            ColumnType::String(len) => DataType::Varchar(len.map(|l| CharacterLength::IntegerLength {
                length: l as u64,
                unit: None,
            })),
            ColumnType::Bool => DataType::Boolean,
            ColumnType::DateTime => DataType::Timestamp(None, sqlparser::ast::TimezoneInfo::None),
            ColumnType::Uuid => DataType::Uuid,
            ColumnType::Json => DataType::JSON,
        };

        ColumnDef {
            name: Ident::new(&col.name),
            data_type,
            collation: None,
            options: vec![], // TODO: Handle NOT NULL, AUTO_INCREMENT, DEFAULT
        }
    }
}
