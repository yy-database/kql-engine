mod common;
use common::assert_sql_eq;
use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::lir::sql_gen::SqlGenerator;
use kql_analyzer::lir::SqlDialect;

#[test]
fn test_aggregate_query() {
    let input = r#"
        namespace db {
            struct Product {
                @primary_key
                id: i32,
                name: string,
                price: f64,
                category: string
            }

            let total_products = Product.count(*);
            let avg_price = Product.avg(price);
            let category_stats = Product.select(category, count(*), sum(price), max(price), min(price));
        }
    "#;

    let mut parser = Parser::new(input);
    let db_ast = parser.parse().unwrap();
    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&db_ast).unwrap();

    let mut mir_gen = MirLowerer::new(hir);
    let mir = mir_gen.lower().unwrap();
    
    let sql_gen = SqlGenerator::new(mir.clone(), SqlDialect::Postgres);
    
    // 1. Test count(*)
    let q1 = mir.queries.get("db::total_products").expect("Query total_products not found");
    let sql1 = sql_gen.generate_mir_query(q1).to_string();
    assert_sql_eq(&sql1, "aggregate_total_products");

    // 2. Test avg(price)
    let q2 = mir.queries.get("db::avg_price").expect("Query avg_price not found");
    let sql2 = sql_gen.generate_mir_query(q2).to_string();
    assert_sql_eq(&sql2, "aggregate_avg_price");

    // 3. Test multiple aggregates
    let q3 = mir.queries.get("db::category_stats").expect("Query category_stats not found");
    let sql3 = sql_gen.generate_mir_query(q3).to_string();
    assert_sql_eq(&sql3, "aggregate_category_stats");
}
