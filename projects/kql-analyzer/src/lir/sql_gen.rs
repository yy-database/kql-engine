use crate::lir::SqlDialect;
use crate::mir::{Column, ColumnType, MirProgram, Table, ReferenceAction};
use sqlparser::ast::{
    CharacterLength, ColumnDef, ColumnOption, ColumnOptionDef, DataType, Ident, ObjectName,
    ReferentialAction, Statement, TableConstraint, Query, SetExpr, Values, Expr, Value,
    Assignment,
};
use sqlparser::tokenizer::Token;

pub struct SqlGenerator {
    pub mir_db: MirProgram,
    pub dialect: SqlDialect,
}

impl SqlGenerator {
    pub fn new(mir_db: MirProgram, dialect: SqlDialect) -> Self {
        Self { mir_db, dialect }
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
            .map(|stmt| format!("{};", stmt))
            .collect()
    }

    pub fn generate_insert(&self, table: &Table) -> Statement {
        let table_name = self.get_table_object_name(table);
        let columns: Vec<Ident> = table.columns.iter()
            .filter(|c| !c.auto_increment) // Skip auto-increment columns for insert
            .map(|c| Ident::new(&c.name))
            .collect();

        let placeholders: Vec<Expr> = (0..columns.len())
            .map(|_| Expr::Value(Value::Placeholder("?".to_string())))
            .collect();

        Statement::Insert {
            or: None,
            into: true,
            table_name,
            columns,
            overwrite: false,
            source: Some(Box::new(Query {
                with: None,
                body: Box::new(SetExpr::Values(Values {
                    explicit_row: false,
                    rows: vec![placeholders],
                })),
                order_by: vec![],
                limit: None,
                offset: None,
                fetch: None,
                locks: vec![],
                for_clause: None,
                limit_by: vec![],
            })),
            partitioned: None,
            after_columns: vec![],
            table: false,
            on: None,
            returning: None,
            replace_into: false,
            priority: None,
            ignore: false,
            table_alias: None,
        }
    }

    pub fn generate_update_by_pk(&self, table: &Table) -> Option<Statement> {
        let pk_cols = table.primary_key.as_ref()?;
        let table_name = self.get_table_object_name(table);
        
        let assignments: Vec<Assignment> = table.columns.iter()
            .filter(|c| !pk_cols.contains(&c.name))
            .map(|c| Assignment {
                id: vec![Ident::new(&c.name)],
                value: Expr::Value(Value::Placeholder("?".to_string())),
            })
            .collect();

        if assignments.is_empty() {
            return None;
        }

        let mut selection = None;
        for pk in pk_cols {
            let condition = Expr::BinaryOp {
                left: Box::new(Expr::Identifier(Ident::new(pk))),
                op: sqlparser::ast::BinaryOperator::Eq,
                right: Box::new(Expr::Value(Value::Placeholder("?".to_string()))),
            };
            selection = match selection {
                Some(existing) => Some(Expr::BinaryOp {
                    left: Box::new(existing),
                    op: sqlparser::ast::BinaryOperator::And,
                    right: Box::new(condition),
                }),
                None => Some(condition),
            };
        }

        Some(Statement::Update {
            table: sqlparser::ast::TableWithJoins {
                relation: sqlparser::ast::TableFactor::Table {
                    name: table_name,
                    alias: None,
                    args: None,
                    with_hints: vec![],
                    version: None,
                    partitions: vec![],
                },
                joins: vec![],
            },
            assignments,
            from: None,
            selection,
            returning: None,
        })
    }

    pub fn generate_delete_by_pk(&self, table: &Table) -> Option<Statement> {
        let pk_cols = table.primary_key.as_ref()?;
        let table_name = self.get_table_object_name(table);

        let mut selection = None;
        for pk in pk_cols {
            let condition = Expr::BinaryOp {
                left: Box::new(Expr::Identifier(Ident::new(pk))),
                op: sqlparser::ast::BinaryOperator::Eq,
                right: Box::new(Expr::Value(Value::Placeholder("?".to_string()))),
            };
            selection = match selection {
                Some(existing) => Some(Expr::BinaryOp {
                    left: Box::new(existing),
                    op: sqlparser::ast::BinaryOperator::And,
                    right: Box::new(condition),
                }),
                None => Some(condition),
            };
        }

        Some(Statement::Delete {
            tables: vec![],
            using: None,
            selection,
            returning: None,
            from: vec![sqlparser::ast::TableWithJoins {
                relation: sqlparser::ast::TableFactor::Table {
                    name: table_name,
                    alias: None,
                    args: None,
                    with_hints: vec![],
                    version: None,
                    partitions: vec![],
                },
                joins: vec![],
            }],
            order_by: vec![],
            limit: None,
        })
    }

    fn get_table_object_name(&self, table: &Table) -> ObjectName {
        let mut name_parts = Vec::new();
        if let Some(schema) = &table.schema {
            if !(self.dialect == SqlDialect::MySql || self.dialect == SqlDialect::Sqlite) || schema != "public" {
                name_parts.push(Ident::new(schema));
            }
        }
        name_parts.push(Ident::new(&table.name));
        ObjectName(name_parts)
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

        for fk in &table.foreign_keys {
            let mut foreign_table_parts = Vec::new();
            if let Some(schema) = &fk.referenced_schema {
                if !(self.dialect == SqlDialect::MySql || self.dialect == SqlDialect::Sqlite) || schema != "public" {
                    foreign_table_parts.push(Ident::new(schema));
                }
            }
            foreign_table_parts.push(Ident::new(&fk.referenced_table));

            constraints.push(TableConstraint::ForeignKey {
                name: Some(Ident::new(&fk.name)),
                columns: fk.columns.iter().map(|c| Ident::new(c)).collect(),
                foreign_table: ObjectName(foreign_table_parts),
                referred_columns: fk.referenced_columns.iter().map(|c| Ident::new(c)).collect(),
                on_delete: fk.on_delete.map(|a| self.map_reference_action(a)),
                on_update: fk.on_update.map(|a| self.map_reference_action(a)),
                characteristics: None,
            });
        }

        Statement::CreateTable {
            or_replace: false,
            temporary: false,
            external: false,
            if_not_exists: true,
            transient: false,
            name: self.get_table_object_name(table),
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
            ColumnType::I16 => DataType::SmallInt(None),
            ColumnType::I32 => DataType::Int(None),
            ColumnType::I64 => DataType::BigInt(None),
            ColumnType::F32 => DataType::Float(None),
            ColumnType::F64 => DataType::Double,
            ColumnType::String(len) => DataType::Varchar(len.map(|l| CharacterLength::IntegerLength {
                length: l as u64,
                unit: None,
            })),
            ColumnType::Bool => DataType::Boolean,
            ColumnType::DateTime => DataType::Timestamp(None, sqlparser::ast::TimezoneInfo::None),
            ColumnType::Uuid => DataType::Uuid,
            ColumnType::Json => DataType::JSON,
            ColumnType::Decimal128 => DataType::Decimal(sqlparser::ast::ExactNumberInfo::None),
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

    fn map_reference_action(&self, action: ReferenceAction) -> ReferentialAction {
        match action {
            ReferenceAction::NoAction => ReferentialAction::NoAction,
            ReferenceAction::Restrict => ReferentialAction::Restrict,
            ReferenceAction::Cascade => ReferentialAction::Cascade,
            ReferenceAction::SetNull => ReferentialAction::SetNull,
            ReferenceAction::SetDefault => ReferentialAction::SetDefault,
        }
    }
}
