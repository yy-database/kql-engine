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

    fn compile_to_json(source: String) -> Result<String, String> {
        let mut compiler = kql_core::Compiler::new();
        compiler.compile(&source).map_err(|e| e.to_string())?;

        serde_json::to_string(&compiler.db).map_err(|e| e.to_string())
    }

    fn compile_to_sql(source: String, dialect: SqlDialect) -> Result<String, String> {
        use kql_analyzer::mir::mir_gen::MirLowerer;
        use kql_analyzer::lir::sql_gen::SqlGenerator;
        use kql_analyzer::lir::SqlDialect as LirDialect;

        let mut compiler = kql_core::Compiler::new();
        compiler.compile(&source).map_err(|e| e.to_string())?;

        // HIR -> MIR
        let mut mir_gen = MirLowerer::new(compiler.db);
        let mir_db = mir_gen.lower().map_err(|e| e.to_string())?;

        // MIR -> LIR (SQL)
        let lir_dialect = match dialect {
            SqlDialect::Postgres => LirDialect::Postgres,
            SqlDialect::Mysql => LirDialect::MySql,
            SqlDialect::Sqlite => LirDialect::Sqlite,
        };

        let sql_gen = SqlGenerator::new(mir_db, lir_dialect);
        let sql_statements = sql_gen.generate_ddl_sql();

        // Join statements with semicolons
        Ok(sql_statements.join(";\n") + ";")
    }
}
