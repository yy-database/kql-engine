use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::codegen::RustGenerator;

#[test]
fn test_rust_codegen_sqlx() {
    let input = r#"
        @schema("public")
        namespace db {
            struct User {
                @primary_key
                id: i32,
                name: string,
                @relation(local: "id", foreign: "user_id")
                posts: [Post]
            }

            struct Post {
                id: i32,
                user_id: i32,
                title: string
            }
        }
    "#;

    let mut parser = Parser::new(input);
    let db_ast = parser.parse().unwrap();
    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&db_ast).unwrap();

    let gen = RustGenerator::new(hir);
    let code = gen.generate();

    println!("{}", code);

    assert!(code.contains("#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]"));
    assert!(code.contains("use sqlx::{FromRow, PgPool, MySqlPool, SqlitePool};"));
    assert!(code.contains("pub struct UserRepository {"));
    assert!(code.contains("pub async fn find(&self, id: i32) -> Result<Option<User>, sqlx::Error> {"));
    assert!(code.contains("pub async fn list(&self) -> Result<Vec<User>, sqlx::Error> {"));
}
