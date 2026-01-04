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
