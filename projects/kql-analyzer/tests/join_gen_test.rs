use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::lir::sql_gen::SqlGenerator;
use kql_analyzer::lir::SqlDialect;

#[test]
fn test_auto_join_generation() {
    let input = r#"
        @schema("public")
        namespace db {
            struct User {
                @primary_key
                id: i32,
                name: string,
                @relation(foreign_key: "id", references: "user_id")
                posts: [Post]
            }

            struct Post {
                @primary_key
                id: i32,
                user_id: i32,
                title: string,
                @relation(foreign_key: "user_id", references: "id")
                author: User
            }
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    
    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&ast).unwrap();
    
    let mut mir_gen = MirLowerer::new(hir);
    let mir = mir_gen.lower().unwrap();
    
    let sql_gen = SqlGenerator::new(mir.clone(), SqlDialect::Postgres);
    
    let post_table = mir.tables.get("db::Post").unwrap();
    let select_sql = sql_gen.generate_select(post_table, &["author"]);
    
    let sql_string = format!("{};", select_sql);
    println!("Generated SQL: {}", sql_string);
    
    assert!(sql_string.contains("SELECT * FROM public.post AS post"));
    assert!(sql_string.contains("LEFT JOIN public.user AS author ON post.user_id = author.id"));
}

#[test]
fn test_many_to_many_join_generation() {
    let input = r#"
        @schema("public")
        namespace db {
            struct User {
                @primary_key
                id: i32,
                name: string,
                @relation(name: "user_roles")
                roles: [Role]
            }

            struct Role {
                @primary_key
                id: i32,
                name: string,
                @relation(name: "user_roles")
                users: [User]
            }
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    
    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&ast).unwrap();
    
    let mut mir_gen = MirLowerer::new(hir);
    let mir = mir_gen.lower().unwrap();
    
    let sql_gen = SqlGenerator::new(mir.clone(), SqlDialect::Postgres);
    
    let user_table = mir.tables.get("db::User").unwrap();
    let select_sql = sql_gen.generate_select(user_table, &["roles"]);
    
    let sql_string = format!("{};", select_sql);
    println!("Generated SQL: {}", sql_string);
    
    // Expected SQL for many-to-many:
    // SELECT * FROM public.user AS user 
    // LEFT JOIN public.user_roles AS user_roles ON user.id = user_roles.user_id
    // LEFT JOIN public.role AS roles ON user_roles.role_id = roles.id
    
    assert!(sql_string.contains("SELECT * FROM public.user AS user"));
    assert!(sql_string.contains("LEFT JOIN public.user_roles AS user_roles ON user.id = user_roles.user_id"));
    assert!(sql_string.contains("LEFT JOIN public.role AS roles ON user_roles.role_id = roles.id"));
}
