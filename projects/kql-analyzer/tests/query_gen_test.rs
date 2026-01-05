use kql_analyzer::hir::{HirProgram, HirStruct, HirField, HirType, HirId, HirExpr, HirExprKind, HirLet, PrimitiveType};
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::lir::sql_gen::SqlGenerator;
use kql_analyzer::lir::SqlDialect;
use kql_types::Span;

#[test]
fn test_auto_join_query_generation() {
    let mut hir = HirProgram::default();
    
    // 1. Define User struct
    let user_id = hir.alloc_id();
    let user_struct = HirStruct {
        id: user_id,
        attrs: vec![],
        name: "User".to_string(),
        namespace: None,
        schema: None,
        fields: vec![
            HirField {
                attrs: vec![],
                name: "id".to_string(),
                ty: HirType::Primitive(PrimitiveType::I32),
                span: Span::default(),
            },
            HirField {
                attrs: vec![],
                name: "name".to_string(),
                ty: HirType::Primitive(PrimitiveType::String),
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };
    
    // 2. Define Post struct with relation to User
    let post_id = hir.alloc_id();
    let post_struct = HirStruct {
        id: post_id,
        attrs: vec![],
        name: "Post".to_string(),
        namespace: None,
        schema: None,
        fields: vec![
            HirField {
                attrs: vec![],
                name: "id".to_string(),
                ty: HirType::Primitive(PrimitiveType::I32),
                span: Span::default(),
            },
            HirField {
                attrs: vec![],
                name: "title".to_string(),
                ty: HirType::Primitive(PrimitiveType::String),
                span: Span::default(),
            },
            HirField {
                attrs: vec![],
                name: "user_id".to_string(),
                ty: HirType::ForeignKey { name: None, entity: user_id },
                span: Span::default(),
            },
            HirField {
                attrs: vec![],
                name: "author".to_string(),
                ty: HirType::Relation {
                    name: None,
                    target: user_id,
                    is_list: false,
                    foreign_key: Some("user_id".to_string()),
                    references: Some("id".to_string()),
                },
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };
    
    hir.structs.insert(user_id, user_struct);
    hir.structs.insert(post_id, post_struct);
    hir.name_to_id.insert("User".to_string(), user_id);
    hir.name_to_id.insert("Post".to_string(), post_id);
    hir.id_to_kind.insert(user_id, kql_analyzer::hir::HirKind::Struct);
    hir.id_to_kind.insert(post_id, kql_analyzer::hir::HirKind::Struct);

    // 3. Define a query let q = Post.author
    // HirExpr for Post.author
    let post_symbol = HirExpr {
        kind: HirExprKind::Symbol("Post".to_string()),
        ty: HirType::Struct(post_id),
        span: Span::default(),
    };
    let author_member = HirExpr {
        kind: HirExprKind::Member {
            object: Box::new(post_symbol),
            member: "author".to_string(),
        },
        ty: HirType::Relation {
            name: None,
            target: user_id,
            is_list: false,
            foreign_key: Some("user_id".to_string()),
            references: Some("id".to_string()),
        },
        span: Span::default(),
    };
    
    let q_let = HirLet {
        id: hir.alloc_id(),
        attrs: vec![],
        name: "q".to_string(),
        namespace: None,
        ty: author_member.ty.clone(),
        value: author_member,
        span: Span::default(),
    };
    hir.lets.insert(q_let.id, q_let);

    // 4. Lower to MIR
    let mut lowerer = MirLowerer::new(hir);
    let mir = lowerer.lower().unwrap();
    
    assert!(mir.queries.contains_key("q"));
    {
        let query = &mir.queries["q"];
        assert_eq!(query.source_table, "post");
        assert_eq!(query.joins.len(), 1);
        assert_eq!(query.joins[0].relation_name, "author");
        assert_eq!(query.joins[0].target_table, "user");
    }

    // 5. Generate SQL
    let sql_gen = SqlGenerator::new(mir, SqlDialect::Postgres);
    let query = &sql_gen.mir_db.queries["q"];
    let sql_stmt = sql_gen.generate_mir_query(query);
    let sql_string = sql_stmt.to_string();
    
    println!("Generated SQL: {}", sql_string);
    assert!(sql_string.contains("FROM post AS post"));
    assert!(sql_string.contains("LEFT JOIN user AS author ON post.user_id = author.id"));
}
