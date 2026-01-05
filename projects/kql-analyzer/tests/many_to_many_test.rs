use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;

#[test]
fn test_many_to_many_lowering() {
    let input = r#"
        struct User {
            id: Key<i32>,
            @relation(name: "user_roles")
            roles: List<Role>,
        }

        struct Role {
            id: Key<i32>,
            @relation(name: "user_roles")
            users: List<User>,
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();

    let mut lowerer = Lowerer::new();
    lowerer.lower_program(&ast).unwrap();
    let hir_db = lowerer.db;

    let mut mir_lowerer = MirLowerer::new(hir_db);
    let mir_db = mir_lowerer.lower().unwrap();

    // Check if User and Role tables exist
    assert!(mir_db.tables.contains_key("User"));
    assert!(mir_db.tables.contains_key("Role"));

    // Check if junction table exists
    let junction_table = mir_db.tables.get("user_roles").expect("Junction table user_roles should exist");
    assert_eq!(junction_table.name, "user_roles");
    assert_eq!(junction_table.columns.len(), 2);
    assert_eq!(junction_table.columns[0].name, "user_id");
    assert_eq!(junction_table.columns[1].name, "role_id");
    
    // Check foreign keys in junction table
    assert_eq!(junction_table.foreign_keys.len(), 2);
    
    let fk1 = &junction_table.foreign_keys[0];
    assert_eq!(fk1.referenced_table, "user");
    assert_eq!(fk1.columns, vec!["user_id"]);
    
    let fk2 = &junction_table.foreign_keys[1];
    assert_eq!(fk2.referenced_table, "role");
    assert_eq!(fk2.columns, vec!["role_id"]);
}
