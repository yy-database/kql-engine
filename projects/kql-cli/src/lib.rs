use clap::{Args, Parser, Subcommand};
use kql_core::Compiler;
use std::path::PathBuf;
use std::fs;
use kql_types::KqlError;

mod cmds;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct KqlApplication {
    #[command(subcommand)]
    command: KqlCommands,
}

#[derive(Subcommand)]
pub enum KqlCommands {
    /// Check a KQL file for syntax and semantic errors
    Check(CheckArgs),
    /// Compile a KQL file and output HIR
    Compile(CompileArgs),
}

#[derive(Args)]
pub struct CheckArgs {
    /// The KQL file to check
    pub file: PathBuf,
}

impl CheckArgs {
    pub async fn run(&self) -> Result<(), KqlError> {
        let source = fs::read_to_string(&self.file)?;
        let mut compiler = Compiler::new();
        compiler.compile(&source)?;
        println!("Check successful!");
        Ok(())
    }
}

#[derive(Args)]
pub struct CompileArgs {
    /// The KQL file to compile
    pub file: PathBuf,
    /// Output HIR as JSON
    #[arg(short, long)]
    pub json: bool,
}

impl CompileArgs {
    pub async fn run(&self) -> Result<(), KqlError> {
        let source = fs::read_to_string(&self.file)?;
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


