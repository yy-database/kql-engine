mod common;
use common::assert_sql_has;
use kql_analyzer::mir::{MirProgram, Table, Column, ColumnType};
use kql_analyzer::migration::{MigrationEngine, MigrationStep};

#[test]
fn test_migration_diff() {
    let mut old_mir = MirProgram::default();
    let old_table = Table {
        schema: Some("public".to_string()),
        name: "users".to_string(),
        columns: vec![
            Column {
                name: "id".to_string(),
                ty: ColumnType::I32,
                nullable: false,
                auto_increment: true,
                default: None,
            },
            Column {
                name: "name".to_string(),
                ty: ColumnType::String(None),
                nullable: false,
                auto_increment: false,
                default: None,
            },
        ],
        primary_key: Some(vec!["id".to_string()]),
        indexes: vec![],
        foreign_keys: vec![],
        relations: vec![],
        lifecycle_hooks: vec![],
        audit: false,
        soft_delete: false,
    };
    old_mir.tables.insert("public::users".to_string(), old_table);

    let mut new_mir = MirProgram::default();
    let new_table = Table {
        schema: Some("public".to_string()),
        name: "users".to_string(),
        columns: vec![
            Column {
                name: "id".to_string(),
                ty: ColumnType::I32,
                nullable: false,
                auto_increment: true,
                default: None,
            },
            Column {
                name: "full_name".to_string(), // renamed name to full_name (will be seen as drop + add for now)
                ty: ColumnType::String(None),
                nullable: false,
                auto_increment: false,
                default: None,
            },
            Column {
                name: "age".to_string(), // new column
                ty: ColumnType::I32,
                nullable: true,
                auto_increment: false,
                default: None,
            },
        ],
        primary_key: Some(vec!["id".to_string()]),
        indexes: vec![],
        foreign_keys: vec![],
        relations: vec![],
        lifecycle_hooks: vec![],
        audit: false,
        soft_delete: false,
    };
    new_mir.tables.insert("public::users".to_string(), new_table);

    let engine = MigrationEngine::new(old_mir, new_mir);
    let steps = engine.diff();

    use kql_analyzer::lir::{SqlGenerator, SqlDialect};
    // Generate SQL
    let gen = SqlGenerator::new(MirProgram::default(), SqlDialect::Postgres);
    let sqls = gen.generate_migration_sql(steps);
    let all_sql = sqls.join(";\n");

    assert_sql_has(&all_sql, &["ADD COLUMN", "age", "DROP COLUMN", "name"]);
}

#[test]
fn test_migration_rename() {
    use kql_analyzer::mir::{Table, Column, ColumnType, MirProgram};
    use kql_analyzer::migration::{MigrationEngine, MigrationStep};
    use kql_analyzer::lir::{SqlGenerator, SqlDialect};

    let mut old_mir = MirProgram::default();
    old_mir.tables.insert("User".to_string(), Table {
        name: "User".to_string(),
        schema: None,
        columns: vec![
            Column { name: "id".to_string(), ty: ColumnType::I32, nullable: false, auto_increment: true, default: None },
            Column { name: "name".to_string(), ty: ColumnType::String(None), nullable: false, auto_increment: false, default: None },
        ],
        primary_key: Some(vec!["id".to_string()]),
        indexes: vec![],
        foreign_keys: vec![],
        relations: vec![],
        lifecycle_hooks: vec![],
        audit: false,
        soft_delete: false,
    });

    let mut new_mir = MirProgram::default();
    new_mir.tables.insert("User".to_string(), Table {
        name: "User".to_string(),
        schema: None,
        columns: vec![
            Column { name: "id".to_string(), ty: ColumnType::I32, nullable: false, auto_increment: true, default: None },
            Column { name: "full_name".to_string(), ty: ColumnType::String(None), nullable: false, auto_increment: false, default: None },
        ],
        primary_key: Some(vec!["id".to_string()]),
        indexes: vec![],
        foreign_keys: vec![],
        relations: vec![],
        lifecycle_hooks: vec![],
        audit: false,
        soft_delete: false,
    });

    let engine = MigrationEngine::new(old_mir, new_mir);
    let steps = engine.diff();
    
    // Manual adjustment for rename detection (not implemented in diff yet)
    let mut adjusted_steps = Vec::new();
    for step in steps {
        match step {
            MigrationStep::AddColumn { table_name, column } if column.name == "full_name" => {
                adjusted_steps.push(MigrationStep::RenameColumn {
                    table_name: table_name.clone(),
                    old_name: "name".to_string(),
                    new_name: "full_name".to_string(),
                });
            }
            MigrationStep::DropColumn { column, .. } if column.name == "name" => {
                // Skip drop
            }
            s => adjusted_steps.push(s),
        }
    }

    let gen = SqlGenerator::new(MirProgram::default(), SqlDialect::Postgres);
    let sqls = gen.generate_migration_sql(adjusted_steps);
    let all_sql = sqls.join(";\n");

    assert_sql_has(&all_sql, &["RENAME COLUMN name TO full_name"]);
}
