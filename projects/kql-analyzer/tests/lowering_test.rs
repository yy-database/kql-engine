mod common;
use common::assert_sql_has;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::lir::sql_gen::SqlGenerator;
use kql_analyzer::lir::SqlDialect;
use kql_parser::parser::Parser;

#[test]
fn test_full_lowering_pipeline() {
    let input = r#"
        struct User {
            @auto_increment
            id: Key<i32>,
            name: String,
            email: String?
        }
    "#;

    // 1. AST
    let mut parser = Parser::new(input);
    let mut ast = Vec::new();
    while !parser.is_eof() {
        ast.push(parser.parse_declaration().unwrap());
    }

    // 2. HIR
    let mut lowerer = Lowerer::new();
    lowerer.lower_decls(ast).unwrap();
    let hir_db = lowerer.db;

    // 3. MIR
    let mut mir_lowerer = MirLowerer::new(hir_db);
    let mir_db = mir_lowerer.lower().unwrap();

    let user_table = mir_db.tables.get("User").unwrap();
    assert_eq!(user_table.name, "user");
    assert_eq!(user_table.columns.len(), 3);
    assert!(user_table.columns[0].auto_increment);
    assert!(user_table.columns[2].nullable);

    // 4. LIR
    let sql_gen = SqlGenerator::new(mir_db, SqlDialect::Postgres);
    let statements = sql_gen.generate_ddl();
    
    assert_eq!(statements.len(), 1);
    let sql = statements[0].to_string();
    assert_sql_has(&sql, &[
        "CREATE TABLE IF NOT EXISTS user",
        "id INT",
        "name VARCHAR"
    ]);
}
