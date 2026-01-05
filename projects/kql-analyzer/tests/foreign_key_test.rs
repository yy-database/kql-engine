use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::mir::ReferenceAction;

#[test]
fn test_foreign_key_lowering() {
    let input = r#"
        struct User {
            id: Key<i32>,
            name: String,
        }

        struct Post {
            id: Key<i32>,
            title: String,
            @relation(on_delete: cascade)
            author_id: Key<User>,
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();

    let mut lowerer = Lowerer::new();
    lowerer.lower_program(&ast).unwrap();
    let hir_db = lowerer.db;

    let mut mir_lowerer = MirLowerer::new(hir_db);
    let mir_db = mir_lowerer.lower().unwrap();

    let post_table = mir_db.tables.get("Post").expect("Post table should exist");
    assert_eq!(post_table.foreign_keys.len(), 1);
    
    let fk = &post_table.foreign_keys[0];
    assert_eq!(fk.columns, vec!["author_id"]);
    assert_eq!(fk.referenced_table, "user");
    assert_eq!(fk.referenced_columns, vec!["id"]);
    assert_eq!(fk.on_delete, Some(ReferenceAction::Cascade));
}
