use kql_ast::{BinaryOpKind, Decl, Expr, LiteralKind};
use kql_parser::{Parser, parser::Precedence};

#[test]
fn test_parse_expression() {
    let source = "1 + 2 * 3";
    let mut parser = Parser::new(source);
    let expr = parser.parse_expression(Precedence::None).unwrap();

    // Simple check to ensure it parsed
    if let Expr::Binary(binary) = expr {
        assert_eq!(binary.op.kind, BinaryOpKind::Add);
    }
    else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_parse_struct() {
    let source = "struct User { id: i32, name: String }";
    let mut parser = Parser::new(source);
    let decl = parser.parse_declaration().unwrap();

    if let Decl::Struct(s) = decl {
        assert_eq!(s.name.name, "User");
        assert_eq!(s.fields.len(), 2);
        assert_eq!(s.fields[0].name.name, "id");
        assert_eq!(s.fields[1].name.name, "name");
    }
    else {
        panic!("Expected struct declaration");
    }
}

#[test]
fn test_parse_enum() {
    let source = "enum Status { Active, Inactive, Pending { message: String } }";
    let mut parser = Parser::new(source);
    let decl = parser.parse_declaration().unwrap();

    if let Decl::Enum(e) = decl {
        assert_eq!(e.name.name, "Status");
        assert_eq!(e.variants.len(), 3);
        assert_eq!(e.variants[0].name.name, "Active");
        assert_eq!(e.variants[2].name.name, "Pending");
        assert!(e.variants[2].fields.is_some());
    }
    else {
        panic!("Expected enum declaration");
    }
}

#[test]
fn test_parse_let() {
    let source = "let x: i32 = 10";
    let mut parser = Parser::new(source);
    let decl = parser.parse_declaration().unwrap();

    if let Decl::Let(l) = decl {
        assert_eq!(l.name.name, "x");
        assert!(l.ty.is_some());
        if let Expr::Literal(lit) = l.value {
            assert!(matches!(lit.kind, LiteralKind::Number(_)));
        }
        else {
            panic!("Expected literal value");
        }
    }
    else {
        panic!("Expected let declaration");
    }
}
