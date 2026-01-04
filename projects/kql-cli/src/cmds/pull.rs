use clap::Args;
use kql_transpiler::Transpiler;
use sqlx::{AnyConnection, Connection, Row};
use kql_types::Result;

#[derive(Args)]
pub struct PullArgs {
    /// Database URL (e.g. sqlite:test.db, postgres://user:pass@localhost/db)
    #[arg(short, long, env = "DATABASE_URL")]
    pub database_url: String,

    /// Output file for the pulled KQL schema
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}

impl PullArgs {
    pub async fn run(&self) -> Result<()> {
        let mut conn = AnyConnection::connect(&self.database_url).await
            .map_err(|e| kql_types::KqlError::database(e.to_string()))?;

        let mut kql_schema = String::new();
        kql_schema.push_str("// Pulled from database\n\n");

        // Simple implementation for SQLite/MySQL/Postgres
        // Note: In a real implementation, we'd use dialect-specific queries
        if self.database_url.starts_with("sqlite") {
            let rows = sqlx::query("SELECT sql FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
                .fetch_all(&mut conn)
                .await
                .map_err(|e| kql_types::KqlError::database(e.to_string()))?;

            for row in rows {
                let sql: String = row.get(0);
                if let Ok(kql) = Transpiler::transpile(&sql) {
                    kql_schema.push_str(&kql);
                    kql_schema.push_str("\n\n");
                }
            }
        } else {
            return Err(kql_types::KqlError::cli("Pull is currently only supported for SQLite in this version."));
        }

        if let Some(output_path) = &self.output {
            std::fs::write(output_path, kql_schema)
                .map_err(|e| kql_types::KqlError::io(e.to_string()))?;
            println!("✓ Schema pulled and saved to {:?}", output_path);
        } else {
            println!("{}", kql_schema);
        }

        Ok(())
    }
}
