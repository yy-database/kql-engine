use sqlparser::dialect::MySqlDialect;
use sqlparser::parser::Parser;
use std::fs;
use std::env;
use std::path::Path;

pub fn assert_sql_eq(actual: &str, expected_file: &str) {
    let dialect = MySqlDialect {};
    
    // Parse actual SQL
    let actual_ast = Parser::parse_sql(&dialect, actual)
        .map_err(|e| format!("Failed to parse actual SQL: {:?}\nSQL: {}", e, actual))
        .expect("Failed to parse actual SQL");
    
    let force_regenerate = env::var("KQL_REGENERATE_TESTS").is_ok() 
        || env::args().any(|arg| arg == "--force-regenerate");

    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("snapshots");
    if !test_dir.exists() {
        fs::create_dir_all(&test_dir).unwrap();
    }
    
    let snapshot_path = test_dir.join(format!("{}.sql", expected_file));
    
    if force_regenerate || !snapshot_path.exists() {
        fs::write(&snapshot_path, actual).expect("Failed to write snapshot");
        return;
    }
    
    let expected = fs::read_to_string(&snapshot_path).expect("Failed to read snapshot");
    let expected_ast = Parser::parse_sql(&dialect, &expected)
        .expect("Failed to parse expected SQL snapshot");
    
    if actual_ast != expected_ast {
        // Fallback to string comparison for better error message if ASTs differ
        // but normalize them first by re-formatting
        let actual_normalized = actual_ast.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(";\n");
        let expected_normalized = expected_ast.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(";\n");
        
        assert_eq!(
            actual_normalized, 
            expected_normalized, 
            "\nSQL mismatch for snapshot: {}\nActual: {}\nExpected: {}", 
            expected_file, actual, expected
        );
    }
}

pub fn assert_sql_has(actual: &str, expected_parts: &[&str]) {
    for part in expected_parts {
        assert!(
            actual.contains(part),
            "\nSQL missing expected part: {}\nActual SQL: {}",
            part,
            actual
        );
    }
}
