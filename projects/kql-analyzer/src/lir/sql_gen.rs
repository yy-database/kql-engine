use crate::mir::{Column, ColumnType, MirDatabase, Table};
use sqlparser::ast::{
    CharacterLength, ColumnDef, ColumnOption, ColumnOptionDef, DataType, Ident, ObjectName,
    Statement, TableConstraint,
};
use sqlparser::tokenizer::Token;

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

    pub fn generate_ddl_sql(&self) -> Vec<String> {
        self.generate_ddl()
            .into_iter()
            .map(|stmt| stmt.to_string())
            .collect()
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

        let mut options = Vec::new();

        if !col.nullable {
            options.push(ColumnOptionDef {
                name: None,
                option: ColumnOption::NotNull,
            });
        }

        if col.auto_increment {
            // Note: Different dialects handle auto increment differently.
            // For now we use a generic DialectPostgres/MySql might need specific handling.
            options.push(ColumnOptionDef {
                name: None,
                option: ColumnOption::DialectSpecific(vec![Token::make_keyword("AUTO_INCREMENT")]),
            });
        }

        if let Some(default_val) = &col.default {
            // Simple default value handling
            options.push(ColumnOptionDef {
                name: None,
                option: ColumnOption::Default(sqlparser::ast::Expr::Value(
                    sqlparser::ast::Value::SingleQuotedString(default_val.clone()),
                )),
            });
        }

        ColumnDef {
            name: Ident::new(&col.name),
            data_type,
            collation: None,
            options,
        }
    }
}
