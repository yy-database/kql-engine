mod common;
use common::assert_sql_eq;
use kql_parser::parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::lir::sql_gen::SqlGenerator;

#[test]
fn test_window_function() {
    let input = r#"
        namespace db {
            struct User {
                @primary_key
                id: i32,
                name: string,
                score: i32,
                city: string
            }

            let user_rank = User.select(
                name,
                score,
                score.count().over(partition_by: city, order_by: score.desc())
            );
        }
    "#;

    let mut parser = Parser::new(input);
    let db = parser.parse().unwrap();

    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&db).unwrap();

    let mut mir_gen = MirLowerer::new(hir);
    let mir = mir_gen.lower().unwrap();

    let sql_gen = SqlGenerator::new(mir.clone(), kql_analyzer::lir::SqlDialect::Postgres);
    
    let q = &mir.queries["db::user_rank"];
    let sql = sql_gen.generate_mir_query(q).to_string();
    
    println!("Generated SQL: {}", sql);

    assert_sql_eq(&sql, "window_function_user_rank");
}
