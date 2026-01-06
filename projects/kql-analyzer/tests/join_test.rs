mod common;
use common::assert_sql_eq;
use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::lir::sql_gen::SqlGenerator;
use kql_analyzer::lir::SqlDialect;

#[test]
fn test_complex_join_query() {
    let input = r#"
        namespace db {
            struct User {
                @primary_key
                id: i32,
                name: string,
                @relation(foreign_key: "author_id")
                posts: [Post]
            }

            struct Post {
                @primary_key
                id: i32,
                title: string,
                author_id: i32,
                @relation(foreign_key: "post_id")
                comments: [Comment]
            }

            struct Comment {
                @primary_key
                id: i32,
                content: string,
                post_id: i32
            }

            let user_posts_with_comments = User.posts.comments.filter(comments.content == "nice");
        }
    "#;

    let mut parser = Parser::new(input);
    let db_ast = parser.parse().unwrap();
    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&db_ast).unwrap();

    let mut mir_gen = MirLowerer::new(hir);
    let mir = mir_gen.lower().unwrap();
    
    let sql_gen = SqlGenerator::new(mir.clone(), SqlDialect::Postgres);
    
    // Check if query exists
    let query = mir.queries.get("db::user_posts_with_comments").expect("Query not found");
    
    let sql_stmt = sql_gen.generate_mir_query(query);
    let sql = sql_stmt.to_string();

    assert_sql_eq(&sql, "join_user_posts_comments_filter");
}
