use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::lir::sql_gen::SqlGenerator;
use kql_analyzer::lir::SqlDialect;
use kql_parser::parser::Parser;

#[test]
fn test_table_annotation() {
    let input = r#"
        @table("users")
        struct User {
            id: Key<i32>,
            name: String,
        }
    "#;

    let mut parser = Parser::new(input);
    let mut ast = Vec::new();
    while !parser.is_eof() {
        ast.push(parser.parse_declaration().unwrap());
    }

    let mut lowerer = Lowerer::new();
    lowerer.lower_decls(ast).unwrap();
    let hir_db = lowerer.db;

    let mut mir_lowerer = MirLowerer::new(hir_db);
    let mir_db = mir_lowerer.lower().unwrap();

    let user_table = mir_db.tables.get("User").expect("Table 'User' not found in MIR");
    assert_eq!(user_table.name, "users");
    assert_eq!(user_table.schema, None);

    let sql_gen = SqlGenerator::new(mir_db, SqlDialect::Postgres);
    let statements = sql_gen.generate_ddl();
    let sql = statements[0].to_string();
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS users"));
}

#[test]
fn test_table_snake_case_default() {
    let input = r#"
        struct UserProfile {
            id: Key<i32>,
            bio: String,
        }
    "#;

    let mut parser = Parser::new(input);
    let mut ast = Vec::new();
    while !parser.is_eof() {
        ast.push(parser.parse_declaration().unwrap());
    }

    let mut lowerer = Lowerer::new();
    lowerer.lower_decls(ast).unwrap();
    let hir_db = lowerer.db;

    let mut mir_lowerer = MirLowerer::new(hir_db);
    let mir_db = mir_lowerer.lower().unwrap();

    let profile_table = mir_db.tables.get("UserProfile").expect("Table 'UserProfile' not found in MIR");
    assert_eq!(profile_table.name, "user_profile");
}

#[test]
fn test_table_schema_named_arg() {
    let input = r#"
        @table(schema: "auth", "users")
        struct User {
            id: Key<i32>,
            name: String,
        }
    "#;

    let mut parser = Parser::new(input);
    let mut ast = Vec::new();
    while !parser.is_eof() {
        ast.push(parser.parse_declaration().unwrap());
    }

    let mut lowerer = Lowerer::new();
    lowerer.lower_decls(ast).unwrap();
    let hir_db = lowerer.db;

    let mut mir_lowerer = MirLowerer::new(hir_db);
    let mir_db = mir_lowerer.lower().unwrap();

    let user_table = mir_db.tables.get("User").expect("Table 'User' not found in MIR");
    assert_eq!(user_table.name, "users");
    assert_eq!(user_table.schema, Some("auth".to_string()));

    let sql_gen = SqlGenerator::new(mir_db, SqlDialect::Postgres);
    let statements = sql_gen.generate_ddl();
    let sql = statements[0].to_string();
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS auth.users"));
}

#[test]
fn test_namespace_validation() {
    // Test: Nested top-level namespace should fail in HIR lowering
    let input = r#"
        namespace mysql {
            namespace pg
        }
    "#;
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let mut lowerer = Lowerer::new();
    let _ = lowerer.lower_program(&ast);
    assert!(!lowerer.errors.is_empty());
    assert!(lowerer.errors.iter().any(|e| e.to_string().contains("Top-level namespace cannot be nested")));

    // Test: Multiple top-level namespaces enforcement in Lowerer (as lint error)
    let input = r#"
        namespace mysql
        struct UserA {}
        namespace pg
        struct UserB {}
    "#;
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    
    let mut lowerer = Lowerer::new();
    let _ = lowerer.lower_program(&ast);
    assert!(!lowerer.errors.is_empty());
    assert!(lowerer.errors.iter().any(|e| e.to_string().contains("Only one top-level namespace is allowed")));
}

#[test]
fn test_toplevel_namespace() {
    let input = r#"
        @schema
        namespace auth
        
        @table("users")
        struct User {
            id: Key<i32>,
            name: String,
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();

    let mut lowerer = Lowerer::new();
    lowerer.lower_program(&ast).unwrap();
    let hir_db = lowerer.db;

    let mut mir_lowerer = MirLowerer::new(hir_db);
    let mir_db = mir_lowerer.lower().unwrap();

    let user_table = mir_db.tables.get("auth::User").expect("Table 'auth::User' not found in MIR");
    assert_eq!(user_table.name, "users");
    assert_eq!(user_table.schema, Some("auth".to_string()));

    let sql_gen = SqlGenerator::new(mir_db, SqlDialect::Postgres);
    let statements = sql_gen.generate_ddl();
    let sql = statements[0].to_string();
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS auth.users"));
}

#[test]
fn test_multiple_errors_collection() {
    let input = r#"
        namespace mysql {
            namespace pg
        }
        namespace redis
        namespace mongo
    "#;
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let mut lowerer = Lowerer::new();
    let _ = lowerer.lower_program(&ast);
    
    // Should have 3 errors:
    // 1. Nested top-level 'pg'
    // 2. Multiple top-level 'redis'
    // 3. Multiple top-level 'mongo'
    assert!(lowerer.errors.len() >= 3);
}

#[test]
fn test_database_block_schema() {
    let input = r#"
        @schema
        namespace auth {
            @table("users")
            struct User {
                id: Key<i32>,
                name: String,
            }
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();

    let mut lowerer = Lowerer::new();
    lowerer.lower_program(&ast).unwrap();
    let hir_db = lowerer.db;

    let mut mir_lowerer = MirLowerer::new(hir_db);
    let mir_db = mir_lowerer.lower().unwrap();

    let user_table = mir_db.tables.get("auth::User").expect("Table 'auth::User' not found in MIR");
    assert_eq!(user_table.name, "users");
    assert_eq!(user_table.schema, Some("auth".to_string()));

    let sql_gen = SqlGenerator::new(mir_db, SqlDialect::Postgres);
    let statements = sql_gen.generate_ddl();
    let sql = statements[0].to_string();
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS auth.users"));
}
