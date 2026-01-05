use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::lir::sql_gen::SqlGenerator;
use kql_analyzer::lir::SqlDialect;

#[test]
fn test_sql_generation() {
    let input = r#"
        @schema("auth_schema")
        namespace auth {
            struct User {
                @auto_increment
                id: Key<i32>
                name: String
                @unique
                email: String
            }

            struct Post {
                id: Key<i32>
                title: String
                author_id: ForeignKey<User>
            }
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    
    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&ast).unwrap();
    
    let mut mir_gen = MirLowerer::new(hir);
    let mir = mir_gen.lower().unwrap();
    
    let sql_gen = SqlGenerator::new(mir, SqlDialect::Postgres);
    let sqls = sql_gen.generate_ddl_sql();
    
    println!("Generated SQLs:");
    for sql in &sqls {
        println!("{}", sql);
    }
    
    // Check User table
    let user_sql = sqls.iter().find(|s| s.contains("auth_schema.user")).expect("User table not found");
    assert!(user_sql.contains("CREATE TABLE IF NOT EXISTS auth_schema.user"));
    assert!(user_sql.contains("id INT NOT NULL"));
    assert!(user_sql.contains("name VARCHAR NOT NULL"));
    assert!(user_sql.contains("email VARCHAR NOT NULL"));
    
    // Check Post table
    let post_sql = sqls.iter().find(|s| s.contains("auth_schema.post")).expect("Post table not found");
    assert!(post_sql.contains("CREATE TABLE IF NOT EXISTS auth_schema.post"));
    assert!(post_sql.contains("author_id INT NOT NULL"));
    assert!(post_sql.contains("CONSTRAINT post_author_id_fk FOREIGN KEY (author_id) REFERENCES auth_schema.user(id)"));
}

#[test]
fn test_aggregation_sql_generation() {
    let input = r#"
        @schema("public")
        struct Product {
            @primary_key
            id: i32,
            price: f64
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    
    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&ast).unwrap();
    
    let mut mir_gen = MirLowerer::new(hir);
    let mir = mir_gen.lower().unwrap();
    
    let sql_gen = SqlGenerator::new(mir.clone(), SqlDialect::Postgres);
    
    // Manual construction of an aggregation expression for testing LIR
    use kql_analyzer::hir::{HirExpr, HirExprKind, HirBinaryOp, HirLiteral};
    use kql_types::Span;
    
    let product_table = mir.tables.get("Product").unwrap();
    
    // Expression: avg(price)
    let price_field = HirExpr {
        kind: HirExprKind::Symbol("price".to_string()),
        ty: kql_analyzer::hir::HirType::Primitive(kql_analyzer::hir::PrimitiveType::F64),
        span: Span::default(),
    };
    let avg_func = HirExpr {
        kind: HirExprKind::Symbol("avg".to_string()),
        ty: kql_analyzer::hir::HirType::Unknown,
        span: Span::default(),
    };
    let avg_call = HirExpr {
        kind: HirExprKind::Call {
            func: Box::new(avg_func),
            args: vec![price_field],
        },
        ty: kql_analyzer::hir::HirType::Primitive(kql_analyzer::hir::PrimitiveType::F64),
        span: Span::default(),
    };
    
    let sql_expr = sql_gen.generate_expr(&avg_call);
    let sql_string = sql_expr.to_string();
    assert_eq!(sql_string, "avg(price)");
}
