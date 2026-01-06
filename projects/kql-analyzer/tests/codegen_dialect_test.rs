use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::codegen::rust::RustGenerator;
use kql_analyzer::lir::SqlDialect;

#[test]
fn test_rust_codegen_dialects() {
    let input = r#"
        @schema("public")
        struct User {
            @primary_key
            id: i32,
            name: string
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    
    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&ast).unwrap();
    
    // Test Postgres
    let pg_gen = RustGenerator::new(hir.clone(), SqlDialect::Postgres);
    let pg_code = pg_gen.generate();
    assert!(pg_code.contains("pool: kql_runtime::KqlPool"));
    assert!(pg_code.contains("WHERE id = $1"));
    assert!(pg_code.contains("INSERT INTO user (id, name) VALUES ($1, $2)"));

    // Test MySql
    let mysql_gen = RustGenerator::new(hir.clone(), SqlDialect::MySql);
    let mysql_code = mysql_gen.generate();
    assert!(mysql_code.contains("pool: kql_runtime::KqlPool"));
    assert!(mysql_code.contains("WHERE id = ?"));
    assert!(mysql_code.contains("INSERT INTO user (id, name) VALUES (?, ?)"));

    // Test Sqlite
    let sqlite_gen = RustGenerator::new(hir.clone(), SqlDialect::Sqlite);
    let sqlite_code = sqlite_gen.generate();
    assert!(sqlite_code.contains("pool: kql_runtime::KqlPool"));
    assert!(sqlite_code.contains("WHERE id = ?"));
    assert!(sqlite_code.contains("INSERT INTO user (id, name) VALUES (?, ?)"));
}
