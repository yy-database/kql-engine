use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::lir::sql_gen::SqlGenerator;
use kql_analyzer::lir::SqlDialect;
use kql_parser::parser::Parser;

#[test]
fn test_composite_pk_and_index() {
    let input = r#"
        @primary_key(tenant_id, user_id)
        @index(email)
        struct User {
            tenant_id: i32,
            user_id: i32,
            email: String,
            name: String
        }
    "#;

    let mut parser = Parser::new(input);
    let mut ast = Vec::new();
    while !parser.is_eof() {
        ast.push(parser.parse_declaration().unwrap());
    }

    let mut lowerer = Lowerer::new();
    lowerer.lower_decls(ast).unwrap();
    let hir_db = lowerer.db;

    let mut mir_lowerer = MirLowerer::new(hir_db);
    let mir_db = mir_lowerer.lower().unwrap();

    let user_table = mir_db.tables.get("User").expect("Table 'User' not found in MIR");
    
    // Check composite PK
    assert_eq!(user_table.primary_key, Some(vec!["tenant_id".to_string(), "user_id".to_string()]));
    
    // Check index
    assert_eq!(user_table.indexes.len(), 1);
    assert_eq!(user_table.indexes[0].columns, vec!["email".to_string()]);
    assert!(!user_table.indexes[0].unique);

    let sql_gen = SqlGenerator::new(mir_db, SqlDialect::Postgres);
    let statements = sql_gen.generate_ddl();
    let sql = statements[0].to_string();
    
    // sqlparser-rs output for CREATE TABLE with composite PK
    assert!(sql.contains("PRIMARY KEY (tenant_id, user_id)"));
}

#[test]
fn test_key_with_entity() {
    let input = r#"
        struct User {
            id: Key<i32>
        }
        struct Post {
            id: Key<i32>,
            author_id: Key<User, i32>
        }
    "#;

    let mut parser = Parser::new(input);
    let mut ast = Vec::new();
    while !parser.is_eof() {
        ast.push(parser.parse_declaration().unwrap());
    }

    let mut lowerer = Lowerer::new();
    lowerer.lower_decls(ast).unwrap();
    let hir_db = lowerer.db;
    
    // Verify HIR for Post.author_id
    let user_id = *hir_db.name_to_id.get("User").unwrap();
    let post_id = *hir_db.name_to_id.get("Post").unwrap();
    let post = hir_db.structs.get(&post_id).unwrap();
    let author_id_field = post.fields.iter().find(|f| f.name == "author_id").unwrap();
    
    if let kql_analyzer::hir::HirType::Key { entity, .. } = &author_id_field.ty {
        assert_eq!(*entity, Some(user_id));
    } else {
        panic!("author_id should be a Key type");
    }
}
