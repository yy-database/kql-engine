mod common;
use common::assert_sql_has;
use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::codegen::RustGenerator;
use kql_analyzer::lir::SqlDialect;

#[test]
fn test_rust_codegen_sqlx() {
    let input = r#"
        @schema("public")
        namespace db {
            struct User {
                @primary_key
                id: i32,
                name: string,
                @relation(foreign_key: "id", references: "user_id")
                posts: [Post]
            }

            struct Post {
                @primary_key
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

    let gen = RustGenerator::new(hir, SqlDialect::Postgres);
    let code = gen.generate();

    assert_sql_has(&code, &[
        "#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]",
        "use sqlx::{FromRow, Postgres, MySql, Sqlite};",
        "pub struct UserRepository {",
        "async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error> {",
        "async fn insert(&self, model: &User) -> Result<(), sqlx::Error> {",
        "async fn update(&self, model: &User) -> Result<(), sqlx::Error> {",
        "async fn delete(&self, id: i32) -> Result<(), sqlx::Error> {",
        "async fn list(&self) -> Result<Vec<User>, sqlx::Error> {",
    ]);
}
