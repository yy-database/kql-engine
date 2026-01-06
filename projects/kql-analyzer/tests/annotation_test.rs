mod common;
use kql_analyzer::hir::lower::Lowerer as HirLowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_parser::Parser;

#[test]
fn test_audit_annotation() {
    let input = r#"
        @audit
        struct User {
            @primary_key
            id: i32
            name: string
        }
    "#;
    let ast = Parser::new(input).parse().unwrap();
    let mut hir_lowerer = HirLowerer::new();
    let hir = hir_lowerer.lower_program(&ast).unwrap();
    let mut mir_lowerer = MirLowerer::new(hir);
    let mir = mir_lowerer.lower().unwrap();
    
    // Check MIR for User table
    let user_table = mir.tables.get("User").unwrap();
    assert!(user_table.audit);
    assert!(user_table.columns.iter().any(|c| c.name == "created_at"));
    assert!(user_table.columns.iter().any(|c| c.name == "updated_at"));
}

#[test]
fn test_soft_delete_annotation() {
    let input = r#"
        @soft_delete
        struct Post {
            @primary_key
            id: i32
            title: string
        }
    "#;
    let ast = Parser::new(input).parse().unwrap();
    let mut hir_lowerer = HirLowerer::new();
    let hir = hir_lowerer.lower_program(&ast).unwrap();
    let mut mir_lowerer = MirLowerer::new(hir);
    let mir = mir_lowerer.lower().unwrap();
    
    // Check MIR for Post table
    let post_table = mir.tables.get("Post").unwrap();
    assert!(post_table.soft_delete);
    assert!(post_table.columns.iter().any(|c| c.name == "deleted_at"));
}

#[test]
fn test_lifecycle_hooks() {
    let input = r#"
        @before_save(validate_user)
        @after_delete(notify_admin)
        struct Profile {
            @primary_key
            id: i32
            bio: string
        }
    "#;
    let ast = Parser::new(input).parse().unwrap();
    let mut hir_lowerer = HirLowerer::new();
    let hir = hir_lowerer.lower_program(&ast).unwrap();
    let mut mir_lowerer = MirLowerer::new(hir);
    let mir = mir_lowerer.lower().unwrap();
    
    // Check MIR for Profile table
    let profile_table = mir.tables.get("Profile").unwrap();
    assert_eq!(profile_table.lifecycle_hooks.len(), 2);
    
    use kql_analyzer::mir::LifecycleEvent;
    
    let before_save = profile_table.lifecycle_hooks.iter().find(|h| h.event == LifecycleEvent::BeforeSave).unwrap();
    assert_eq!(before_save.function, "validate_user");
    
    let after_delete = profile_table.lifecycle_hooks.iter().find(|h| h.event == LifecycleEvent::AfterDelete).unwrap();
    assert_eq!(after_delete.function, "notify_admin");
}

#[test]
fn test_annotation_sql_generation() {
    let input = r#"
        @audit
        @soft_delete
        struct Product {
            @primary_key
            id: i32
            name: string
        }
    "#;
    let ast = Parser::new(input).parse().unwrap();
    let mut hir_lowerer = HirLowerer::new();
    let hir = hir_lowerer.lower_program(&ast).unwrap();
    let mut mir_lowerer = MirLowerer::new(hir);
    let mir = mir_lowerer.lower().unwrap();
    
    use kql_analyzer::lir::{SqlGenerator, SqlDialect};
    
    let sql_gen = SqlGenerator::new(mir, SqlDialect::Postgres);
    let statements = sql_gen.generate_ddl();
    let sql = statements[0].to_string();
    
    common::assert_sql_has(&sql, &[
        "created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP",
        "updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP",
        "deleted_at TIMESTAMP",
    ]);
}
