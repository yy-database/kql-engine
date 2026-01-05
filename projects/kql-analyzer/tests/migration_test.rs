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
    };
    new_mir.tables.insert("public::users".to_string(), new_table);

    let engine = MigrationEngine::new(old_mir, new_mir);
    let steps = engine.diff();

    assert!(!steps.is_empty());
    
    let mut has_drop_name = false;
    let mut has_add_full_name = false;
    let mut has_add_age = false;

    for step in steps {
        match step {
            MigrationStep::DropColumn { column_name, .. } if column_name == "name" => has_drop_name = true,
            MigrationStep::AddColumn { column, .. } if column.name == "full_name" => has_add_full_name = true,
            MigrationStep::AddColumn { column, .. } if column.name == "age" => has_add_age = true,
            _ => {}
        }
    }

    assert!(has_drop_name);
    assert!(has_add_full_name);
    assert!(has_add_age);
}
