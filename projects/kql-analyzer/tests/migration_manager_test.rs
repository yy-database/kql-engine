use kql_analyzer::migration::manager::MigrationManager;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_migration_creation() {
    let dir = tempdir().unwrap();
    let manager = MigrationManager::new(dir.path());

    let sqls = vec![
        "CREATE TABLE users (id INT PRIMARY KEY);".to_string(),
        "ALTER TABLE users ADD COLUMN name TEXT;".to_string(),
    ];

    let file_path = manager.create_migration("init_db", &sqls).unwrap();
    
    assert!(file_path.exists());
    let filename = file_path.file_name().unwrap().to_str().unwrap();
    assert!(filename.contains("init_db"));
    assert!(filename.ends_with(".sql"));

    let content = fs::read_to_string(file_path).unwrap();
    assert!(content.contains("CREATE TABLE users"));
    assert!(content.contains("ALTER TABLE users"));
}

#[test]
fn test_list_migrations() {
    let dir = tempdir().unwrap();
    let manager = MigrationManager::new(dir.path());

    manager.create_migration("first", &["SELECT 1;".to_string()]).unwrap();
    manager.create_migration("second", &["SELECT 2;".to_string()]).unwrap();

    let migrations = manager.list_migrations().unwrap();
    assert_eq!(migrations.len(), 2);
    
    // Check sorting
    let name1 = migrations[0].file_name().unwrap().to_str().unwrap();
    let name2 = migrations[1].file_name().unwrap().to_str().unwrap();
    assert!(name1 < name2);
}
