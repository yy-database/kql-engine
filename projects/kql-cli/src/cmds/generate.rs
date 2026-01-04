use clap::Args;
use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::codegen::RustGenerator;
use kql_types::Result;
use std::path::PathBuf;

#[derive(Args)]
pub struct GenerateArgs {
    /// The KQL file to generate code from
    pub input: PathBuf,
    /// The output directory for generated code
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// Language to generate (currently only 'rust')
    #[arg(short, long, default_value = "rust")]
    pub lang: String,
}

impl GenerateArgs {
    pub fn run(&self) -> Result<()> {
        let content = std::fs::read_to_string(&self.input)
            .map_err(|e| kql_types::KqlError::io(e.to_string()))?;
        
        let mut parser = Parser::new(&content);
        let ast = parser.parse()?;
        
        let mut lowerer = Lowerer::new();
        let hir = lowerer.lower_database(&ast)?;
        
        if self.lang == "rust" {
            let generator = RustGenerator::new(hir);
            let code = generator.generate();
            
            if let Some(output_path) = &self.output {
                std::fs::write(output_path, code)
                    .map_err(|e| kql_types::KqlError::io(e.to_string()))?;
                println!("Generated Rust models to {}", output_path.display());
            } else {
                println!("{}", code);
            }
        } else {
            return Err(kql_types::KqlError::cli(format!("Unsupported language: {}", self.lang)));
        }
        
        Ok(())
    }
}
