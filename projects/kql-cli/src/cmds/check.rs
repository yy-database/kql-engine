use clap::Args;
use kql_core::Compiler;
use kql_types::KqlError;
use std::path::PathBuf;
use tokio::fs;

#[derive(Args)]
pub struct CheckArgs {
    /// The KQL file to check
    pub file: PathBuf,
}

impl CheckArgs {
    pub async fn run(&self) -> Result<(), KqlError> {
        let source = fs::read_to_string(&self.file).await.map_err(|e| KqlError::internal(e.to_string()))?;
        let mut compiler = Compiler::new();
        compiler.compile(&source)?;
        println!("Check successful!");
        Ok(())
    }
}
