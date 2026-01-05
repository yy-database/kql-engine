use kql_analyzer::hir::lower::Lowerer;
use kql_parser::parser::Parser;
use kql_analyzer::hir::{HirType, PrimitiveType};

#[test]
fn test_member_access_lowering() {
    let input = r#"
        struct User {
            id: i32,
            name: String,
        }

        let user: User? = null
        let id = user.id
        let name = user.name
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();

    let mut lowerer = Lowerer::new();
    lowerer.lower_program(&ast).unwrap();
    let db = lowerer.db;

    let id_let = db.lets.values().find(|l| l.name == "id").unwrap();
    assert_eq!(id_let.ty, HirType::Optional(Box::new(HirType::Primitive(PrimitiveType::I32))));

    let name_let = db.lets.values().find(|l| l.name == "name").unwrap();
    assert_eq!(name_let.ty, HirType::Optional(Box::new(HirType::Primitive(PrimitiveType::String))));
}

#[test]
fn test_enum_member_access_lowering() {
    let input = r#"
        enum Role {
            Admin,
            User,
        }

        let admin = Role.Admin
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();

    let mut lowerer = Lowerer::new();
    lowerer.lower_program(&ast).unwrap();
    let db = lowerer.db;

    let admin_let = db.lets.values().find(|l| l.name == "admin").unwrap();
    if let HirType::Enum(_) = admin_let.ty {
        // OK
    } else {
        panic!("Expected Enum type for 'admin', found {:?}", admin_let.ty);
    }
}
