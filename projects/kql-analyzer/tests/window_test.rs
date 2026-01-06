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
    
    let mut sql_queries = std::collections::HashMap::new();
    for (name, query) in &mir.queries {
        let stmt = sql_gen.generate_mir_query(query);
        sql_queries.insert(name.clone(), format!("{}", stmt));
    }

    let sql = &sql_queries["db::user_rank"];
    println!("Generated SQL: {}", sql);

    // Verify SQL contains OVER clause
    assert!(sql.contains("SELECT"));
    assert!(sql.contains("name"));
    assert!(sql.contains("score"));
    assert!(sql.contains("count(user.score) OVER (PARTITION BY user.city ORDER BY user.score DESC)") || 
            sql.contains("count(score) OVER (PARTITION BY city ORDER BY score DESC)"));
}
