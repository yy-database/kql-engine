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
    let result = lowerer.lower_program(&ast);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Top-level namespace cannot be nested"));

    // Test: Single top-level namespace enforcement in Parser
    // Actually, our parser just stops after the first top-level namespace,
    // so a second one would just be ignored (if it's brace-less).
    let input = r#"
        namespace mysql
        struct UserA {}
        namespace pg
        struct UserB {}
    "#;
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    // The parser consumes everything after 'namespace mysql' into mysql's decls.
    // So 'namespace pg' and 'struct UserB' are inside 'mysql'.
    // Then 'mysql' is a top-level brace-less namespace, which is nested (effectively).
    // Wait, let's trace:
    // 1. parse() sees 'namespace mysql'. is_block = false.
    // 2. It enters the 'while !self.is_eof()' loop at line 75 in parser.rs.
    // 3. It parses 'struct UserA {}'.
    // 4. It parses 'namespace pg'. is_block = false.
    // 5. It parses 'struct UserB {}'.
    // 6. Loop ends.
    // So AST is Database { decls: [ Namespace { name: "mysql", decls: [ Struct(UserA), Namespace(pg), Struct(UserB) ] } ] }
    // When lowering, the inner 'namespace pg' will trigger the "Top-level namespace cannot be nested" error.
    let mut lowerer = Lowerer::new();
    let result = lowerer.lower_program(&ast);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Top-level namespace cannot be nested"));
}

#[test]
fn test_toplevel_namespace() {
    let input = r#"
        @schema("auth")
        namespace Auth
        
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

    let user_table = mir_db.tables.get("Auth::User").expect("Table 'Auth::User' not found in MIR");
    assert_eq!(user_table.name, "users");
    assert_eq!(user_table.schema, Some("auth".to_string()));

    let sql_gen = SqlGenerator::new(mir_db, SqlDialect::Postgres);
    let statements = sql_gen.generate_ddl();
    let sql = statements[0].to_string();
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS auth.users"));
}

#[test]
fn test_database_block_schema() {
    let input = r#"
        @schema("auth")
        namespace Auth {
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

    let user_table = mir_db.tables.get("Auth::User").expect("Table 'Auth::User' not found in MIR");
    assert_eq!(user_table.name, "users");
    assert_eq!(user_table.schema, Some("auth".to_string()));

    let sql_gen = SqlGenerator::new(mir_db, SqlDialect::Postgres);
    let statements = sql_gen.generate_ddl();
    let sql = statements[0].to_string();
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS auth.users"));
}
