use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::hir::HirType;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::mir::ColumnType;

#[test]
fn test_type_alias_lowering() {
    let input = r#"
        type MyInt = i32;
        type MyString = String;
        
        struct User {
            id: MyInt,
            name: MyString,
        }
    "#;
    
    let mut parser = Parser::new(input);
    let ast_db = parser.parse().expect("Failed to parse");
    
    let mut lowerer = Lowerer::new();
    let hir_db = lowerer.lower_program(&ast_db).expect("Failed to lower");
    
    // Check if User fields have the resolved types
    let user_id = hir_db.name_to_id.get("User").expect("User struct not found");
    let user_struct = hir_db.structs.get(user_id).expect("User struct data not found");
    
    let id_field = user_struct.fields.iter().find(|f| f.name == "id").expect("id field not found");
    let name_field = user_struct.fields.iter().find(|f| f.name == "name").expect("name field not found");
    
    // The lowered type should be the aliased type
    assert!(matches!(id_field.ty, HirType::Primitive(kql_analyzer::hir::PrimitiveType::I32)));
    assert!(matches!(name_field.ty, HirType::Primitive(kql_analyzer::hir::PrimitiveType::String)));
}

#[test]
fn test_nested_type_alias() {
    let input = r#"
        namespace Core {
            type ID = i64;
        }
        
        struct Post {
            id: Core::ID,
            title: String,
        }
    "#;
    
    let mut parser = Parser::new(input);
    let ast_db = parser.parse().expect("Failed to parse");
    
    let mut lowerer = Lowerer::new();
    let hir_db = lowerer.lower_program(&ast_db).expect("Failed to lower");
    
    let post_id = hir_db.name_to_id.get("Post").expect("Post struct not found");
    let post_struct = hir_db.structs.get(post_id).expect("Post struct data not found");
    
    let id_field = post_struct.fields.iter().find(|f| f.name == "id").expect("id field not found");
    assert!(matches!(id_field.ty, HirType::Primitive(kql_analyzer::hir::PrimitiveType::I64)));
}

#[test]
fn test_recursive_type_alias() {
    let input = r#"
        type A = B;
        type B = A;
        
        struct S {
            x: A,
        }
    "#;
    
    let mut parser = Parser::new(input);
    let ast_db = parser.parse().expect("Failed to parse");
    
    let mut lowerer = Lowerer::new();
    let res = lowerer.lower_program(&ast_db);
    
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.to_string().contains("Recursive type alias detected"));
}

#[test]
fn test_layout_annotation() {
    let input = r#"
        @layout(u8)
        enum Status {
            Active,
            Inactive,
        }
        
        @layout(json)
        struct Meta {
            key: String,
            value: String,
        }

        struct Profile {
            id: Key<i32>,
            meta: Meta,
        }
    "#;
    
    let mut parser = Parser::new(input);
    let ast_db = parser.parse().expect("Failed to parse");
    
    let mut lowerer = Lowerer::new();
    let hir_db = lowerer.lower_program(&ast_db).expect("Failed to lower");
    
    let status_id = hir_db.name_to_id.get("Status").expect("Status enum not found");
    let status_enum = hir_db.enums.get(status_id).expect("Status enum data not found");
    assert!(matches!(status_enum.layout, Some(HirType::Primitive(kql_analyzer::hir::PrimitiveType::U8))));
    
    let meta_id = hir_db.name_to_id.get("Meta").expect("Meta struct not found");
    let meta_struct = hir_db.structs.get(meta_id).expect("Meta struct data not found");
    assert!(matches!(meta_struct.layout, Some(kql_analyzer::hir::StructLayout::Json)));

    // Check MIR
    let mut mir_lowerer = MirLowerer::new(hir_db);
    let mir_db = mir_lowerer.lower().expect("Failed to lower to MIR");

    // Meta struct should NOT have a table
    assert!(mir_db.tables.get("Meta").is_none());

    // Profile table should have a JSON column for 'meta'
    let profile_table = mir_db.tables.get("Profile").expect("Profile table not found");
    let meta_col = profile_table.columns.iter().find(|c| c.name == "meta").expect("meta column not found");
    assert_eq!(meta_col.ty, ColumnType::Json);
}

#[test]
fn test_mir_json_column() {
    let input = r#"
        @layout(json)
        struct Config {
            key: String,
            value: String,
        }

        struct System {
            id: Key<i32>,
            config: Config,
        }
    "#;
    let mut parser = Parser::new(input);
    let ast_db = parser.parse().expect("Failed to parse");
    let mut lowerer = Lowerer::new();
    let hir_db = lowerer.lower_program(&ast_db).expect("Failed to lower to HIR");
    
    let mut mir_lowerer = MirLowerer::new(hir_db);
    let mir_db = mir_lowerer.lower().expect("Failed to lower to MIR");

    let system_table = mir_db.tables.get("System").expect("System table not found");
    let config_col = system_table.columns.iter().find(|c| c.name == "config").expect("config column not found");
    assert_eq!(config_col.ty, ColumnType::Json);
    
    // Config should not be a table
    assert!(mir_db.tables.get("Config").is_none());
}
