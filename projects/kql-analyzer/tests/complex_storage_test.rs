use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::codegen::RustGenerator;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::lir::sql_gen::SqlGenerator;
use kql_analyzer::lir::SqlDialect;

#[test]
fn test_complex_storage_codegen() {
    let input = r#"
        @schema("public")
        namespace db {
            @layout(json)
            struct Metadata {
                version: string,
                tags: [string]
            }

            enum Status {
                Active,
                Inactive,
                Suspended { reason: string, until: string }
            }

            struct User {
                @primary_key
                id: i32,
                name: string,
                meta: Metadata,
                history: [string],
                status: Status
            }
        }
    "#;

    let mut parser = Parser::new(input);
    let db_ast = parser.parse().unwrap();
    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&db_ast).unwrap();

    // 1. Test Rust Codegen
    let rust_gen = RustGenerator::new(hir.clone(), SqlDialect::Postgres);
    let rust_code = rust_gen.generate();
    println!("--- Rust Code ---\n{}", rust_code);

    // Verify #[sqlx(json)] attributes
    assert!(rust_code.contains("#[sqlx(json)]"));
    assert!(rust_code.contains("pub meta: Metadata,"));
    assert!(rust_code.contains("pub history: Vec<String>,"));
    assert!(rust_code.contains("pub status: Status,"));

    // Verify derived traits for non-table structs/enums
    assert!(rust_code.contains("#[derive(Debug, Clone, Serialize, Deserialize)]\n    pub struct Metadata {"));
    assert!(rust_code.contains("#[derive(Debug, Clone, Serialize, Deserialize)]\n    pub enum Status {"));

    // 2. Test Migration SQL Codegen
    let mut mir_gen = MirLowerer::new(hir);
    let mir = mir_gen.lower().unwrap();
    
    let sql_gen = SqlGenerator::new(mir.clone(), SqlDialect::Postgres);
    let ddl = sql_gen.generate_ddl_sql();
    let ddl_str = ddl.join("\n");
    println!("--- SQL DDL ---\n{}", ddl_str);

    // Verify JSONB columns in Postgres
    assert!(ddl_str.contains("meta JSONB NOT NULL"));
    assert!(ddl_str.contains("history JSONB NOT NULL"));
    assert!(ddl_str.contains("status JSONB NOT NULL"));

    // Verify Metadata struct does NOT have its own table
    assert!(!ddl_str.contains("CREATE TABLE metadata"));
}
