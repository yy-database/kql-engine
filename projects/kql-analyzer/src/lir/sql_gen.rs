use crate::hir::{
    HirBinaryOp, HirExpr, HirExprKind, HirLiteral, HirUnaryOp,
};
use crate::lir::SqlDialect;
use crate::mir::{Column, ColumnType, MirProgram, Table, ReferenceAction, mir_gen::to_snake_case};
use sqlparser::ast::{
    CharacterLength, ColumnDef, ColumnOption, ColumnOptionDef, DataType, Ident, ObjectName,
    ReferentialAction, Statement, TableConstraint, Query, SetExpr, Values, Expr, Value,
    Assignment, Select, SelectItem, TableWithJoins, TableFactor, Join, JoinOperator, JoinConstraint,
    AlterTableOperation, BinaryOperator, UnaryOperator, Function, FunctionArg, FunctionArgExpr,
};
use sqlparser::tokenizer::Token;
use crate::migration::MigrationStep;

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

    pub fn generate_migration(&self, steps: Vec<MigrationStep>) -> Vec<Statement> {
        let mut statements = Vec::new();
        for step in steps {
            match step {
                MigrationStep::CreateTable(table) => {
                    statements.push(self.generate_create_table(&table));
                }
                MigrationStep::DropTable(table) => {
                    statements.push(Statement::Drop {
                        object_type: sqlparser::ast::ObjectType::Table,
                        if_exists: true,
                        names: vec![self.get_table_object_name(&table)],
                        cascade: false,
                        restrict: false,
                        purge: false,
                        temporary: false,
                    });
                }
                MigrationStep::AddColumn { table_name, column } => {
                    statements.push(Statement::AlterTable {
                        name: ObjectName(vec![Ident::new(table_name)]),
                        if_exists: true,
                        only: false,
                        operations: vec![AlterTableOperation::AddColumn {
                            column_keyword: true,
                            if_not_exists: true,
                            column_def: self.generate_column_def(&column),
                        }],
                    });
                }
                MigrationStep::DropColumn { table_name, column } => {
                    statements.push(Statement::AlterTable {
                        name: ObjectName(vec![Ident::new(table_name)]),
                        if_exists: true,
                        only: false,
                        operations: vec![AlterTableOperation::DropColumn {
                            column_name: Ident::new(column.name),
                            if_exists: true,
                            cascade: false,
                        }],
                    });
                }
                MigrationStep::AlterColumn { table_name, old_column: _, new_column } => {
                    // This is dialect specific. For now, a simple version:
                    let column_name = Ident::new(new_column.name.clone());
                    let data_type = self.generate_column_def(&new_column).data_type;
                    statements.push(Statement::AlterTable {
                        name: ObjectName(vec![Ident::new(table_name)]),
                        if_exists: true,
                        only: false,
                        operations: vec![AlterTableOperation::AlterColumn {
                            column_name,
                            op: sqlparser::ast::AlterColumnOperation::SetDataType {
                                data_type,
                                using: None,
                            },
                        }],
                    });
                }
                MigrationStep::RenameTable { old_name, new_name } => {
                    statements.push(Statement::AlterTable {
                        name: ObjectName(vec![Ident::new(old_name)]),
                        if_exists: true,
                        only: false,
                        operations: vec![AlterTableOperation::RenameTable {
                            table_name: ObjectName(vec![Ident::new(new_name)]),
                        }],
                    });
                }
                MigrationStep::RenameColumn { table_name, old_name, new_name } => {
                    statements.push(Statement::AlterTable {
                        name: ObjectName(vec![Ident::new(table_name)]),
                        if_exists: true,
                        only: false,
                        operations: vec![AlterTableOperation::RenameColumn {
                            old_column_name: Ident::new(old_name),
                            new_column_name: Ident::new(new_name),
                        }],
                    });
                }
                MigrationStep::AddIndex { table_name, index } => {
                    statements.push(Statement::CreateIndex {
                        name: Some(ObjectName(vec![Ident::new(&index.name)])),
                        table_name: ObjectName(vec![Ident::new(table_name)]),
                        columns: index.columns.iter().map(|c| sqlparser::ast::OrderByExpr {
                            expr: Expr::Identifier(Ident::new(c)),
                            asc: None,
                            nulls_first: None,
                        }).collect(),
                        unique: index.unique,
                        if_not_exists: true,
                        using: None,
                        include: vec![],
                        nulls_distinct: None,
                        predicate: None,
                        concurrently: false,
                    });
                }
                MigrationStep::DropIndex { table_name: _, index } => {
                    statements.push(Statement::Drop {
                        object_type: sqlparser::ast::ObjectType::Index,
                        if_exists: true,
                        names: vec![ObjectName(vec![Ident::new(index.name)])],
                        cascade: false,
                        restrict: false,
                        purge: false,
                        temporary: false,
                    });
                }
                MigrationStep::AddForeignKey { table_name, foreign_key } => {
                    let mut foreign_table_parts = Vec::new();
                    if let Some(schema) = &foreign_key.referenced_schema {
                        if !(self.dialect == SqlDialect::MySql || self.dialect == SqlDialect::Sqlite) || schema != "public" {
                            foreign_table_parts.push(Ident::new(schema));
                        }
                    }
                    foreign_table_parts.push(Ident::new(&foreign_key.referenced_table));

                    statements.push(Statement::AlterTable {
                        name: ObjectName(vec![Ident::new(table_name)]),
                        if_exists: true,
                        only: false,
                        operations: vec![AlterTableOperation::AddConstraint(TableConstraint::ForeignKey {
                            name: Some(Ident::new(&foreign_key.name)),
                            columns: foreign_key.columns.iter().map(|c| Ident::new(c)).collect(),
                            foreign_table: ObjectName(foreign_table_parts),
                            referred_columns: foreign_key.referenced_columns.iter().map(|c| Ident::new(c)).collect(),
                            on_delete: foreign_key.on_delete.map(|a| self.map_reference_action(a)),
                            on_update: foreign_key.on_update.map(|a| self.map_reference_action(a)),
                            characteristics: None,
                        })],
                    });
                }
                MigrationStep::DropForeignKey { table_name, foreign_key } => {
                    statements.push(Statement::AlterTable {
                        name: ObjectName(vec![Ident::new(table_name)]),
                        if_exists: true,
                        only: false,
                        operations: vec![AlterTableOperation::DropConstraint {
                            name: Ident::new(foreign_key.name),
                            if_exists: true,
                            cascade: false,
                        }],
                    });
                }
            }
        }
        statements
    }

    pub fn generate_migration_sql(&self, steps: Vec<MigrationStep>) -> Vec<String> {
        self.generate_migration(steps)
            .into_iter()
            .map(|stmt| format!("{};", stmt))
            .collect()
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

    pub fn generate_expr(&self, expr: &HirExpr) -> Expr {
        match &expr.kind {
            HirExprKind::Literal(lit) => match lit {
                HirLiteral::Integer64(n) => Expr::Value(Value::Number(n.to_string(), false)),
                HirLiteral::Float64(f) => Expr::Value(Value::Number(f.to_string(), false)),
                HirLiteral::String(s) => Expr::Value(Value::SingleQuotedString(s.clone())),
                HirLiteral::Bool(b) => Expr::Value(Value::Boolean(*b)),
                HirLiteral::Null => Expr::Value(Value::Null),
            },
            HirExprKind::Symbol(s) => Expr::Identifier(Ident::new(s)),
            HirExprKind::Member { object, member } => {
                let obj_expr = self.generate_expr(object);
                if let Expr::Identifier(id) = obj_expr {
                    Expr::CompoundIdentifier(vec![id, Ident::new(member)])
                } else {
                    // Fallback or error handling
                    Expr::Identifier(Ident::new(member))
                }
            }
            HirExprKind::Binary { left, op, right } => {
                let sql_op = match op {
                    HirBinaryOp::Add => BinaryOperator::Plus,
                    HirBinaryOp::Sub => BinaryOperator::Minus,
                    HirBinaryOp::Mul => BinaryOperator::Multiply,
                    HirBinaryOp::Div => BinaryOperator::Divide,
                    HirBinaryOp::Eq => BinaryOperator::Eq,
                    HirBinaryOp::NotEq => BinaryOperator::NotEq,
                    HirBinaryOp::Lt => BinaryOperator::Lt,
                    HirBinaryOp::LtEq => BinaryOperator::LtEq,
                    HirBinaryOp::Gt => BinaryOperator::Gt,
                    HirBinaryOp::GtEq => BinaryOperator::GtEq,
                    HirBinaryOp::And => BinaryOperator::And,
                    HirBinaryOp::Or => BinaryOperator::Or,
                    HirBinaryOp::Mod => BinaryOperator::Modulo,
                };
                Expr::BinaryOp {
                    left: Box::new(self.generate_expr(left)),
                    op: sql_op,
                    right: Box::new(self.generate_expr(right)),
                }
            }
            HirExprKind::Unary { op, expr } => {
                let sql_op = match op {
                    HirUnaryOp::Not => UnaryOperator::Not,
                    HirUnaryOp::Neg => UnaryOperator::Minus,
                };
                Expr::UnaryOp {
                    op: sql_op,
                    expr: Box::new(self.generate_expr(expr)),
                }
            }
            HirExprKind::Cast { expr, target_ty } => {
                // Generate a CAST(expr AS target_ty)
                let data_type = match target_ty {
                    crate::hir::HirType::Primitive(p) => match p {
                        crate::hir::PrimitiveType::I8 => DataType::Custom(ObjectName(vec![Ident::new("TINYINT")]), vec![]),
                        crate::hir::PrimitiveType::I16 => DataType::SmallInt(None),
                        crate::hir::PrimitiveType::I32 => DataType::Int(None),
                        crate::hir::PrimitiveType::I64 => DataType::BigInt(None),
                        crate::hir::PrimitiveType::U8 => DataType::Custom(ObjectName(vec![Ident::new("TINYINT UNSIGNED")]), vec![]),
                        crate::hir::PrimitiveType::U16 => DataType::Custom(ObjectName(vec![Ident::new("SMALLINT UNSIGNED")]), vec![]),
                        crate::hir::PrimitiveType::U32 => DataType::Custom(ObjectName(vec![Ident::new("INT UNSIGNED")]), vec![]),
                        crate::hir::PrimitiveType::U64 => DataType::Custom(ObjectName(vec![Ident::new("BIGINT UNSIGNED")]), vec![]),
                        crate::hir::PrimitiveType::F32 => DataType::Float(None),
                        crate::hir::PrimitiveType::F64 => DataType::Double,
                        crate::hir::PrimitiveType::String => DataType::Varchar(None),
                        crate::hir::PrimitiveType::Bool => DataType::Boolean,
                        crate::hir::PrimitiveType::DateTime => DataType::Timestamp(None, sqlparser::ast::TimezoneInfo::None),
                        crate::hir::PrimitiveType::Uuid => DataType::Uuid,
                        crate::hir::PrimitiveType::D64 => DataType::Decimal(sqlparser::ast::ExactNumberInfo::None),
                        crate::hir::PrimitiveType::D128 => DataType::Decimal(sqlparser::ast::ExactNumberInfo::None),
                    },
                    _ => DataType::Varchar(None), // Fallback
                };
                Expr::Cast {
                    expr: Box::new(self.generate_expr(expr)),
                    data_type,
                    format: None,
                }
            }
            HirExprKind::Call { func, args } => {
                if let HirExprKind::Symbol(name) = &func.kind {
                    let sql_args = args.iter().map(|arg| {
                        FunctionArg::Unnamed(FunctionArgExpr::Expr(self.generate_expr(arg)))
                    }).collect();
                    
                    Expr::Function(Function {
                        name: ObjectName(vec![Ident::new(name)]),
                        args: sql_args,
                        over: None,
                        distinct: false,
                        special: false,
                        order_by: vec![],
                        filter: None,
                        null_treatment: None,
                    })
                } else {
                    Expr::Value(Value::Null) // Fallback
                }
            }
            _ => Expr::Value(Value::Null), // Unimplemented
        }
    }

    pub fn generate_mir_query(&self, query: &crate::mir::MirQuery) -> Statement {
        let table = self.mir_db.tables.values().find(|t| t.name == query.source_table)
            .expect("Source table not found");
        
        let table_name = self.get_table_object_name(table);
        let table_alias = table.name.clone();

        let mut joins = Vec::new();
        for mir_join in &query.joins {
            joins.push(self.generate_mir_join(&table_alias, mir_join));
        }

        let projection = if query.projection.is_empty() || matches!(query.projection[0], crate::mir::MirProjection::All) {
            vec![SelectItem::Wildcard(sqlparser::ast::WildcardAdditionalOptions::default())]
        } else {
            query.projection.iter().map(|p| self.generate_mir_projection(p)).collect()
        };

        let selection = query.selection.as_ref().map(|e| self.generate_mir_expr(e));

        Statement::Query(Box::new(Query {
            with: None,
            body: Box::new(SetExpr::Select(Box::new(Select {
                distinct: None,
                top: None,
                projection,
                into: None,
                from: vec![TableWithJoins {
                    relation: TableFactor::Table {
                        name: table_name,
                        alias: Some(sqlparser::ast::TableAlias {
                            name: Ident::new(&table_alias),
                            columns: vec![],
                        }),
                        args: None,
                        with_hints: vec![],
                        version: None,
                        partitions: vec![],
                    },
                    joins,
                }],
                lateral_views: vec![],
                selection,
                group_by: sqlparser::ast::GroupByExpr::Expressions(vec![]),
                cluster_by: vec![],
                distribute_by: vec![],
                sort_by: vec![],
                having: None,
                named_window: vec![],
                qualify: None,
            }))),
            order_by: vec![],
            limit: None,
            offset: None,
            fetch: None,
            locks: vec![],
            for_clause: None,
            limit_by: vec![],
        }))
    }

    fn generate_mir_join(&self, source_alias: &str, join: &crate::mir::MirJoin) -> Join {
        let target_table = self.mir_db.tables.values().find(|t| t.name == join.target_table)
            .expect("Target table not found");
        
        let target_table_name = self.get_table_object_name(target_table);
        let target_alias = join.relation_name.clone();

        // Find the relation metadata to get join columns
        let source_table = self.mir_db.tables.values().find(|t| t.name == source_alias || t.name == to_snake_case(source_alias))
            .expect("Source table not found for join");
        
        let rel = source_table.relations.iter().find(|r| r.name == join.relation_name)
            .expect("Relation not found in source table");

        let condition = Expr::BinaryOp {
            left: Box::new(Expr::CompoundIdentifier(vec![
                Ident::new(source_alias),
                Ident::new(&rel.foreign_key_column),
            ])),
            op: BinaryOperator::Eq,
            right: Box::new(Expr::CompoundIdentifier(vec![
                Ident::new(&target_alias),
                Ident::new(&rel.target_column),
            ])),
        };

        let join_operator = match join.join_type {
            crate::mir::MirJoinType::Inner => JoinOperator::Inner(JoinConstraint::On(condition)),
            crate::mir::MirJoinType::Left => JoinOperator::LeftOuter(JoinConstraint::On(condition)),
            crate::mir::MirJoinType::Right => JoinOperator::RightOuter(JoinConstraint::On(condition)),
            crate::mir::MirJoinType::Full => JoinOperator::FullOuter(JoinConstraint::On(condition)),
        };

        Join {
            relation: TableFactor::Table {
                name: target_table_name,
                alias: Some(sqlparser::ast::TableAlias {
                    name: Ident::new(&target_alias),
                    columns: vec![],
                }),
                args: None,
                with_hints: vec![],
                version: None,
                partitions: vec![],
            },
            join_operator,
        }
    }

    fn generate_mir_projection(&self, proj: &crate::mir::MirProjection) -> SelectItem {
        match proj {
            crate::mir::MirProjection::All => SelectItem::Wildcard(sqlparser::ast::WildcardAdditionalOptions::default()),
            crate::mir::MirProjection::Field(name) => SelectItem::UnnamedExpr(Expr::Identifier(Ident::new(name))),
            crate::mir::MirProjection::Alias(alias, expr) => SelectItem::ExprWithAlias {
                expr: self.generate_mir_expr(expr),
                alias: Ident::new(alias),
            },
            crate::mir::MirProjection::Aggregation(agg) => {
                let func_name = agg.func.clone();
                let arg = self.generate_mir_expr(&agg.arg);
                let func = Function {
                    name: ObjectName(vec![Ident::new(func_name)]),
                    args: vec![FunctionArg::Unnamed(FunctionArgExpr::Expr(arg))],
                    over: None,
                    distinct: false,
                    special: false,
                    order_by: vec![],
                    filter: None,
                    null_treatment: None,
                };
                let expr = Expr::Function(func);
                if let Some(alias) = &agg.alias {
                    SelectItem::ExprWithAlias {
                        expr,
                        alias: Ident::new(alias),
                    }
                } else {
                    SelectItem::UnnamedExpr(expr)
                }
            }
        }
    }

    fn generate_mir_expr(&self, expr: &crate::mir::MirExpr) -> Expr {
        match expr {
            crate::mir::MirExpr::Column { table_alias, column } => {
                if let Some(alias) = table_alias {
                    Expr::CompoundIdentifier(vec![Ident::new(alias), Ident::new(column)])
                } else {
                    Expr::Identifier(Ident::new(column))
                }
            }
            crate::mir::MirExpr::Literal(lit) => match lit {
                crate::mir::MirLiteral::Integer64(n) => Expr::Value(Value::Number(n.to_string(), false)),
                crate::mir::MirLiteral::Float64(f) => Expr::Value(Value::Number(f.to_string(), false)),
                crate::mir::MirLiteral::String(s) => Expr::Value(Value::SingleQuotedString(s.clone())),
                crate::mir::MirLiteral::Bool(b) => Expr::Value(Value::Boolean(*b)),
                crate::mir::MirLiteral::Null => Expr::Value(Value::Null),
            },
            crate::mir::MirExpr::Binary { left, op, right } => Expr::BinaryOp {
                left: Box::new(self.generate_mir_expr(left)),
                op: self.map_mir_binary_op(*op),
                right: Box::new(self.generate_mir_expr(right)),
            },
            crate::mir::MirExpr::Unary { op, expr } => Expr::UnaryOp {
                op: self.map_mir_unary_op(*op),
                expr: Box::new(self.generate_mir_expr(expr)),
            },
            crate::mir::MirExpr::Call { func, args } => {
                Expr::Function(Function {
                    name: ObjectName(vec![Ident::new(func)]),
                    args: args.iter().map(|a| FunctionArg::Unnamed(FunctionArgExpr::Expr(self.generate_mir_expr(a)))).collect(),
                    over: None,
                    distinct: false,
                    special: false,
                    order_by: vec![],
                    filter: None,
                    null_treatment: None,
                })
            }
        }
    }

    fn map_mir_binary_op(&self, op: crate::mir::MirBinaryOp) -> BinaryOperator {
        match op {
            crate::mir::MirBinaryOp::Add => BinaryOperator::Plus,
            crate::mir::MirBinaryOp::Sub => BinaryOperator::Minus,
            crate::mir::MirBinaryOp::Mul => BinaryOperator::Multiply,
            crate::mir::MirBinaryOp::Div => BinaryOperator::Divide,
            crate::mir::MirBinaryOp::Mod => BinaryOperator::Modulo,
            crate::mir::MirBinaryOp::Eq => BinaryOperator::Eq,
            crate::mir::MirBinaryOp::NotEq => BinaryOperator::NotEq,
            crate::mir::MirBinaryOp::Gt => BinaryOperator::Gt,
            crate::mir::MirBinaryOp::Lt => BinaryOperator::Lt,
            crate::mir::MirBinaryOp::GtEq => BinaryOperator::GtEq,
            crate::mir::MirBinaryOp::LtEq => BinaryOperator::LtEq,
            crate::mir::MirBinaryOp::And => BinaryOperator::And,
            crate::mir::MirBinaryOp::Or => BinaryOperator::Or,
        }
    }

    fn map_mir_unary_op(&self, op: crate::mir::MirUnaryOp) -> UnaryOperator {
        match op {
            crate::mir::MirUnaryOp::Neg => UnaryOperator::Minus,
            crate::mir::MirUnaryOp::Not => UnaryOperator::Not,
        }
    }

    pub fn generate_select(&self, table: &Table, relations: &[&str]) -> Statement {
        let table_name = self.get_table_object_name(table);
        let table_alias = table.name.clone();

        let mut joins = Vec::new();
        let projection = vec![SelectItem::Wildcard(
            sqlparser::ast::WildcardAdditionalOptions::default(),
        )];

        for rel_name in relations {
            if let Some(rel) = table.relations.iter().find(|r| &r.name == rel_name) {
                // Check if this is a many-to-many relation
                if let Some(rn) = &rel.relation_name {
                    if let Some(junction_table) = self.mir_db.tables.get(rn) {
                        // Many-to-Many Join
                        let junction_table_name = self.get_table_object_name(junction_table);
                        let junction_alias = rn.clone();
                        
                        // Find the column in junction table that points to current table
                        let source_fk_col = format!("{}_id", table.name.to_lowercase());
                        let target_fk_col = format!("{}_id", rel.target_table.to_lowercase());

                        // 1. Join junction table
                        joins.push(Join {
                            relation: TableFactor::Table {
                                name: junction_table_name,
                                alias: Some(sqlparser::ast::TableAlias {
                                    name: Ident::new(&junction_alias),
                                    columns: vec![],
                                }),
                                args: None,
                                with_hints: vec![],
                                version: None,
                                partitions: vec![],
                            },
                            join_operator: JoinOperator::LeftOuter(JoinConstraint::On(Expr::BinaryOp {
                                left: Box::new(Expr::CompoundIdentifier(vec![
                                    Ident::new(&table_alias),
                                    Ident::new(&rel.foreign_key_column),
                                ])),
                                op: sqlparser::ast::BinaryOperator::Eq,
                                right: Box::new(Expr::CompoundIdentifier(vec![
                                    Ident::new(&junction_alias),
                                    Ident::new(&source_fk_col),
                                ])),
                            })),
                        });

                        // 2. Join target table
                        if let Some(target_table) = self.mir_db.tables.values().find(|t| t.name == rel.target_table) {
                            let target_table_name = self.get_table_object_name(target_table);
                            let target_alias = rel.name.clone();

                            joins.push(Join {
                                relation: TableFactor::Table {
                                    name: target_table_name,
                                    alias: Some(sqlparser::ast::TableAlias {
                                        name: Ident::new(&target_alias),
                                        columns: vec![],
                                    }),
                                    args: None,
                                    with_hints: vec![],
                                    version: None,
                                    partitions: vec![],
                                },
                                join_operator: JoinOperator::LeftOuter(JoinConstraint::On(Expr::BinaryOp {
                                    left: Box::new(Expr::CompoundIdentifier(vec![
                                        Ident::new(&junction_alias),
                                        Ident::new(&target_fk_col),
                                    ])),
                                    op: sqlparser::ast::BinaryOperator::Eq,
                                    right: Box::new(Expr::CompoundIdentifier(vec![
                                        Ident::new(&target_alias),
                                        Ident::new(&rel.target_column),
                                    ])),
                                })),
                            });
                        }
                        continue;
                    }
                }

                // Regular One-to-Many / One-to-One Join
                if let Some(target_table) = self.mir_db.tables.values().find(|t| t.name == rel.target_table) {
                    let target_table_name = self.get_table_object_name(target_table);
                    let target_alias = rel.name.clone();

                    joins.push(Join {
                        relation: TableFactor::Table {
                            name: target_table_name,
                            alias: Some(sqlparser::ast::TableAlias {
                                name: Ident::new(&target_alias),
                                columns: vec![],
                            }),
                            args: None,
                            with_hints: vec![],
                            version: None,
                            partitions: vec![],
                        },
                        join_operator: JoinOperator::LeftOuter(JoinConstraint::On(Expr::BinaryOp {
                            left: Box::new(Expr::CompoundIdentifier(vec![
                                Ident::new(&table_alias),
                                Ident::new(&rel.foreign_key_column),
                            ])),
                            op: sqlparser::ast::BinaryOperator::Eq,
                            right: Box::new(Expr::CompoundIdentifier(vec![
                                Ident::new(&target_alias),
                                Ident::new(&rel.target_column),
                            ])),
                        })),
                    });
                }
            }
        }

        Statement::Query(Box::new(Query {
            with: None,
            body: Box::new(SetExpr::Select(Box::new(Select {
                distinct: None,
                top: None,
                projection,
                into: None,
                from: vec![TableWithJoins {
                    relation: TableFactor::Table {
                        name: table_name,
                        alias: Some(sqlparser::ast::TableAlias {
                            name: Ident::new(&table_alias),
                            columns: vec![],
                        }),
                        args: None,
                        with_hints: vec![],
                        version: None,
                        partitions: vec![],
                    },
                    joins,
                }],
                lateral_views: vec![],
                selection: None,
                group_by: sqlparser::ast::GroupByExpr::Expressions(vec![]),
                cluster_by: vec![],
                distribute_by: vec![],
                sort_by: vec![],
                having: None,
                named_window: vec![],
                qualify: None,
            }))),
            order_by: vec![],
            limit: None,
            offset: None,
            fetch: None,
            locks: vec![],
            for_clause: None,
            limit_by: vec![],
        }))
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
            ColumnType::I8 => DataType::Custom(ObjectName(vec![Ident::new("TINYINT")]), vec![]),
            ColumnType::I16 => DataType::SmallInt(None),
            ColumnType::I32 => DataType::Int(None),
            ColumnType::I64 => DataType::BigInt(None),
            ColumnType::U8 => DataType::Custom(ObjectName(vec![Ident::new("TINYINT UNSIGNED")]), vec![]),
            ColumnType::U16 => DataType::Custom(ObjectName(vec![Ident::new("SMALLINT UNSIGNED")]), vec![]),
            ColumnType::U32 => DataType::Custom(ObjectName(vec![Ident::new("INT UNSIGNED")]), vec![]),
            ColumnType::U64 => DataType::Custom(ObjectName(vec![Ident::new("BIGINT UNSIGNED")]), vec![]),
            ColumnType::F32 => DataType::Float(None),
            ColumnType::F64 => DataType::Double,
            ColumnType::String(len) => DataType::Varchar(len.map(|l| CharacterLength::IntegerLength {
                length: l as u64,
                unit: None,
            })),
            ColumnType::Bool => DataType::Boolean,
            ColumnType::DateTime => DataType::Timestamp(None, sqlparser::ast::TimezoneInfo::None),
            ColumnType::Uuid => DataType::Uuid,
            ColumnType::Json => {
                if self.dialect == SqlDialect::Postgres {
                    DataType::Custom(ObjectName(vec![Ident::new("JSONB")]), vec![])
                } else {
                    DataType::JSON
                }
            },
            ColumnType::Decimal64 => DataType::Decimal(sqlparser::ast::ExactNumberInfo::None),
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
}
