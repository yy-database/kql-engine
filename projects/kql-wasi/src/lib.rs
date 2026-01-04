wit_bindgen::generate!({
    path: "compiler.wit",
    world: "compiler",
    exports: {
        world: KqlCompiler,
    },
});

struct KqlCompiler;

impl Guest for KqlCompiler {
    fn check(source: String) -> Result<(), String> {
        let mut compiler = kql_core::Compiler::new();
        compiler.compile(&source).map_err(|e| e.to_string())
    }

    fn compile(source: String) -> Result<String, String> {
        let mut compiler = kql_core::Compiler::new();
        compiler.compile(&source).map_err(|e| e.to_string())?;

        // For now, return names as a simple string
        let names: Vec<_> = compiler.db.name_to_id.keys().cloned().collect();
        Ok(format!("Compiled names: {:?}", names))
    }
}
