use kql_analyzer::hir::{HirProgram, lower::Lowerer};
use kql_parser::Parser;
use kql_types::Result;

pub struct Compiler {
    pub db: HirProgram,
}

impl Compiler {
    pub fn new() -> Self {
        Self { db: HirProgram::default() }
    }

    pub fn compile(&mut self, source: &str) -> Result<()> {
        let mut parser = Parser::new(source);
        let mut decls = Vec::new();

        // Simple loop to parse all declarations in the file
        while !parser.is_eof() {
            decls.push(parser.parse_declaration()?);
        }

        let mut lowerer = Lowerer { db: std::mem::take(&mut self.db) };

        lowerer.lower_decls(decls)?;
        self.db = lowerer.db;

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
}
