use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::hir::{HirExprKind, HirLiteral, PrimitiveType, HirType};
use kql_parser::parser::Parser;

#[test]
fn test_type_inference_basic() {
    let input = r#"
        let x: i32 = 10
        let y: f64 = 20.5
        let z = x + y
        let s = "hello"
        let b = true
    "#;

    let mut parser = Parser::new(input);
    let mut ast = Vec::new();
    while !parser.is_eof() {
        if let Ok(decl) = parser.parse_declaration() {
            ast.push(decl);
        } else {
            break;
        }
    }

    let mut lowerer = Lowerer::new();
    lowerer.lower_decls(ast).unwrap();
    let db = lowerer.db;

    // Check x
    let x_id = db.name_to_id.get("x").unwrap();
    let x_let = db.lets.get(x_id).unwrap();
    assert_eq!(x_let.ty, HirType::Primitive(PrimitiveType::I32));

    // Check y
    let y_id = db.name_to_id.get("y").unwrap();
    let y_let = db.lets.get(y_id).unwrap();
    assert_eq!(y_let.ty, HirType::Primitive(PrimitiveType::F64));

    // Check z
    let z_id = db.name_to_id.get("z").unwrap();
    let z_let = db.lets.get(z_id).unwrap();
    assert_eq!(z_let.ty, HirType::Primitive(PrimitiveType::F64)); // i32 + f64 -> f64
}

#[test]
fn test_builtin_function_inference() {
    let input = r#"
        let t = now()
        let u = uuid()
        let c = count()
    "#;

    let mut parser = Parser::new(input);
    let mut ast = Vec::new();
    while !parser.is_eof() {
        if let Ok(decl) = parser.parse_declaration() {
            ast.push(decl);
        } else {
            break;
        }
    }

    let mut lowerer = Lowerer::new();
    lowerer.lower_decls(ast).unwrap();
    let db = lowerer.db;

    // Check t
    let t_id = db.name_to_id.get("t").unwrap();
    let t_let = db.lets.get(t_id).unwrap();
    assert_eq!(t_let.ty, HirType::Primitive(PrimitiveType::DateTime));

    // Check u
    let u_id = db.name_to_id.get("u").unwrap();
    let u_let = db.lets.get(u_id).unwrap();
    assert_eq!(u_let.ty, HirType::Primitive(PrimitiveType::Uuid));

    // Check c
    let c_id = db.name_to_id.get("c").unwrap();
    let c_let = db.lets.get(c_id).unwrap();
    assert_eq!(c_let.ty, HirType::Primitive(PrimitiveType::I64));
}
