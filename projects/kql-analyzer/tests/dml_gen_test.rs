mod common;
use common::{assert_sql_eq, assert_sql_has};
use kql_analyzer::lir::{SqlGenerator, SqlDialect};
use kql_analyzer::mir::{MirProgram, Table, Column, ColumnType};

#[test]
fn test_insert_generation() {
    let mut mir = MirProgram::default();
    let table = Table {
        schema: Some("auth".to_string()),
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
    };
    mir.tables.insert("auth::users".to_string(), table.clone());

    let gen = SqlGenerator::new(mir, SqlDialect::Postgres);
    let insert = gen.generate_insert(&table);
    let sql = format!("{};", insert);
    
    // id should be skipped because it's auto_increment
    assert_sql_has(&sql, &["INSERT INTO auth.users (name) VALUES (?)"]);
}

#[test]
fn test_update_by_pk_generation() {
    let mut mir = MirProgram::default();
    let table = Table {
        schema: Some("auth".to_string()),
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
    };
    mir.tables.insert("auth::users".to_string(), table.clone());

    let gen = SqlGenerator::new(mir, SqlDialect::Postgres);
    let update = gen.generate_update_by_pk(&table).unwrap();
    let sql = format!("{};", update);
    
    assert_sql_has(&sql, &["UPDATE auth.users SET name = ? WHERE id = ?"]);
}

#[test]
fn test_delete_by_pk_generation() {
    let mut mir = MirProgram::default();
    let table = Table {
        schema: Some("auth".to_string()),
        name: "users".to_string(),
        columns: vec![
            Column {
                name: "id".to_string(),
                ty: ColumnType::I32,
                nullable: false,
                auto_increment: true,
                default: None,
            },
        ],
        primary_key: Some(vec!["id".to_string()]),
        indexes: vec![],
        foreign_keys: vec![],
        relations: vec![],
    };
    mir.tables.insert("auth::users".to_string(), table.clone());

    let gen = SqlGenerator::new(mir, SqlDialect::Postgres);
    let delete = gen.generate_delete_by_pk(&table).unwrap();
    let sql = format!("{};", delete);
    
    assert_sql_has(&sql, &["DELETE FROM auth.users WHERE id = ?"]);
}
