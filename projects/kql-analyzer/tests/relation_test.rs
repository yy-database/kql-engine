mod common;
use common::assert_sql_has;
use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::lir::sql_gen::SqlGenerator;
use kql_analyzer::lir::SqlDialect;

#[test]
fn test_one_to_many_relation() {
    let input = r#"
        namespace social {
            struct User {
                id: Key<i32>
                name: String
                @relation(name: "user_posts", foreign_key: "author_id")
                posts: [Post]
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
    
    // Check MIR
    let user_table = mir.tables.get("social::User").expect("User table not found");
    let post_table = mir.tables.get("social::Post").expect("Post table not found");
    
    // User table should NOT have 'posts' column (it's virtual)
    assert!(!user_table.columns.iter().any(|c| c.name == "posts"));
    
    // User table should have a relation named 'posts'
    let rel = user_table.relations.iter().find(|r| r.name == "posts").expect("Relation 'posts' not found");
    assert_eq!(rel.target_table, "post");
    assert_eq!(rel.foreign_key_column, "author_id");
    
    // SQL generation should only produce 2 tables
    let sql_gen = SqlGenerator::new(mir, SqlDialect::Postgres);
    let sqls = sql_gen.generate_ddl_sql();
    assert_eq!(sqls.len(), 2);
}

#[test]
fn test_many_to_many_relation() {
    let input = r#"
        namespace social {
            struct User {
                id: Key<i32>
                name: String
                @relation(name: "user_groups")
                groups: [Group]
            }

            struct Group {
                id: Key<i32>
                name: String
                @relation(name: "user_groups")
                members: [User]
            }
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    
    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&ast).unwrap();
    
    let mut mir_gen = MirLowerer::new(hir);
    let mir = mir_gen.lower().unwrap();
    
    // Should have 3 tables: user, group, and the junction table 'user_groups'
    assert!(mir.tables.contains_key("social::User"));
    assert!(mir.tables.contains_key("social::Group"));
    assert!(mir.tables.contains_key("user_groups"));
    
    let junction = mir.tables.get("user_groups").unwrap();
    assert_eq!(junction.name, "user_groups");
    assert_eq!(junction.columns.len(), 2);
    assert!(junction.columns.iter().any(|c| c.name == "user_id"));
    assert!(junction.columns.iter().any(|c| c.name == "group_id"));
    
    let sql_gen = SqlGenerator::new(mir, SqlDialect::Postgres);
    let sqls = sql_gen.generate_ddl_sql();
    
    println!("Generated SQLs for Many-to-Many:");
    for sql in &sqls {
        println!("{}", sql);
    }
    
    assert_eq!(sqls.len(), 3);
    let junction_sql = sqls.iter().find(|s| s.contains("CREATE TABLE IF NOT EXISTS user_groups")).expect("Junction table SQL not found");
    assert_sql_has(junction_sql, &[
        "user_id INT NOT NULL",
        "group_id INT NOT NULL",
        "PRIMARY KEY (user_id, group_id)"
    ]);
}
