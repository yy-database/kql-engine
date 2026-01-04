use clap::Args;
use kql_core::Compiler;
use std::path::PathBuf;
use tokio::fs;
use kql_types::Result;

#[derive(Args)]
pub struct CheckArgs {
    /// The KQL file to check
    pub file: PathBuf,
}

impl CheckArgs {
    pub async fn run(&self) -> Result<()> {
        let source = fs::read_to_string(&self.file).await?;
        let mut compiler = Compiler::new();
        compiler.compile(&source)?;
        println!("Check successful!");
        Ok(())
    }
}
