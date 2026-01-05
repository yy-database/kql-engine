use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::codegen::rust::RustGenerator;
use kql_parser::parser::Parser;

#[test]
fn test_rust_codegen_with_namespaces() {
    let input = r#"
        namespace auth {
            struct User {
                id: Key<i32>,
                name: String,
            }

            namespace internal {
                enum Role {
                    Admin,
                    User,
                }
            }
        }

        struct GlobalConfig {
            version: String,
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();

    let mut lowerer = Lowerer::new();
    lowerer.lower_program(&ast).unwrap();
    let hir_db = lowerer.db;

    let generator = RustGenerator::new(hir_db);
    let code = generator.generate();

    println!("{}", code);

    assert!(code.contains("pub struct GlobalConfig {"));
    assert!(code.contains("pub mod auth {"));
    assert!(code.contains("pub struct User {"));
    assert!(code.contains("pub mod internal {"));
    assert!(code.contains("pub enum Role {"));
    assert!(code.contains("use super::*;"));
}
