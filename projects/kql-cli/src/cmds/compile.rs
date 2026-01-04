use clap::Args;
use kql_core::Compiler;
use std::path::PathBuf;
use tokio::fs;
use kql_types::Result;

#[derive(Args)]
pub struct CompileArgs {
    /// The KQL file to compile
    pub file: PathBuf,
    /// Output HIR as JSON
    #[arg(short, long)]
    pub json: bool,
}

impl CompileArgs {
    pub async fn run(&self) -> Result<()> {
        let source = fs::read_to_string(&self.file).await?;
        let mut compiler = Compiler::new();
        compiler.compile(&source)?;

        if self.json {
            let json_output = serde_json::to_string_pretty(&compiler.db)?;
            println!("{}", json_output);
        } else {
            println!("Compilation successful!");
            println!("Structures: {}", compiler.db.structs.len());
            println!("Enums: {}", compiler.db.enums.len());
            println!("Variables: {}", compiler.db.lets.len());
        }
        Ok(())
    }
}
