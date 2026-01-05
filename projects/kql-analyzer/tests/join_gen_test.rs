use kql_analyzer::mir::{MirProgram, Table, Column, ColumnType, Relation};
use kql_analyzer::lir::sql_gen::SqlGenerator;
use kql_analyzer::lir::SqlDialect;

#[test]
fn test_auto_join_generation() {
    let mut mir = MirProgram::default();
    
    // User table
    let user_table = Table {
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
        relations: vec![
            Relation {
                name: "profile".to_string(),
                foreign_key_column: "id".to_string(),
                target_table: "profiles".to_string(),
                target_column: "user_id".to_string(),
            }
        ],
    };
    mir.tables.insert("public::users".to_string(), user_table.clone());

    // Profile table
    let profile_table = Table {
        schema: Some("public".to_string()),
        name: "profiles".to_string(),
        columns: vec![
            Column {
                name: "id".to_string(),
                ty: ColumnType::I32,
                nullable: false,
                auto_increment: true,
                default: None,
            },
            Column {
                name: "user_id".to_string(),
                ty: ColumnType::I32,
                nullable: false,
                auto_increment: false,
                default: None,
            },
            Column {
                name: "bio".to_string(),
                ty: ColumnType::String(None),
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
    mir.tables.insert("public::profiles".to_string(), profile_table);

    let gen = SqlGenerator::new(mir, SqlDialect::Postgres);
    let select = gen.generate_select(&user_table, &["profile"]);
    let sql = format!("{};", select);
    
    println!("Generated SQL: {}", sql);
    
    assert!(sql.contains("SELECT * FROM public.users AS users"));
    assert!(sql.contains("LEFT JOIN public.profiles AS profile ON users.id = profile.user_id"));
}
