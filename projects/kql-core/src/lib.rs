use kql_analyzer::hir::{HirProgram, lower::Lowerer};
use kql_parser::Parser;
use kql_types::Result;

pub mod config;
pub use config::KqlConfig;

pub struct Compiler {
    pub db: HirProgram,
}

impl Compiler {
    pub fn new() -> Self {
        Self { db: HirProgram::default() }
    }

    pub fn compile(&mut self, source: &str) -> Result<()> {
        let mut parser = Parser::new(source);
        let ast = parser.parse()?;

        let mut lowerer = Lowerer::new();
        lowerer.db = std::mem::take(&mut self.db);

        let res = lowerer.lower_program(&ast);
        self.db = lowerer.db;

        if !lowerer.errors.is_empty() {
            // Return the first error for now, or we could return a combined error
            return Err(lowerer.errors[0].clone());
        }

        res?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_basic() {
        let source = "
            struct User {
                id: i32,
                name: String
            }
            let x: i32 = 10
        ";
        let mut compiler = Compiler::new();
        compiler.compile(source).unwrap();

        assert_eq!(compiler.db.name_to_id.len(), 2);
        assert!(compiler.db.name_to_id.contains_key("User"));
        assert!(compiler.db.name_to_id.contains_key("x"));
    }

    #[test]
    fn test_config_serialization() {
        let config = KqlConfig {
            project: config::ProjectConfig {
                name: "test-project".to_string(),
                version: "1.0.0".to_string(),
            },
            database: config::DatabaseConfig {
                url: "postgres://localhost/test".to_string(),
                dialect: "postgres".to_string(),
            },
            codegen: config::CodegenConfig {
                out_dir: "src/gen".into(),
                language: "rust".to_string(),
            },
        };

        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("name = \"test-project\""));
    }
}
