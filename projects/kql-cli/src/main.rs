use clap::Parser;
use kql_cli::{KqlApplication, KqlCommands};

#[tokio::main]
async fn main() -> kql_types::Result<()> {
    let cli = KqlApplication::parse();

    match cli.command {
        KqlCommands::Check(args) => args.run().await?,
        KqlCommands::Compile(args) => args.run().await?,
        KqlCommands::Migrate(args) => args.run().await?,
        KqlCommands::Pull(args) => args.run().await?,
    }

    Ok(())
}
