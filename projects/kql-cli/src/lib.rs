use clap::{Parser, Subcommand};

mod cmds;

pub use cmds::{CheckArgs, CompileArgs};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct KqlApplication {
    #[command(subcommand)]
    pub command: KqlCommands,
}

#[derive(Subcommand)]
pub enum KqlCommands {
    /// Check a KQL file for syntax and semantic errors
    Check(CheckArgs),
    /// Compile a KQL file and output HIR
    Compile(CompileArgs),
}
