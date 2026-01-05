use clap::{Args, Subcommand};
use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::mir::MirProgram;
use kql_analyzer::lir::SqlDialect;
use kql_analyzer::lir::sql_gen::SqlGenerator;
use kql_analyzer::migration::{MigrationEngine, manager::MigrationManager};
use kql_types::Result;
use std::path::PathBuf;
use sqlx::AnyConnection;
use sqlx::Connection;

#[derive(Args)]
pub struct MigrationArgs {
    #[command(subcommand)]
    pub command: MigrationCommand,
}

#[derive(Subcommand)]
pub enum MigrationCommand {
    /// Generate a new migration file by comparing KQL with previous state
    Generate {
        /// Name of the migration
        #[arg(short, long)]
        name: String,

        /// The KQL schema file
        #[arg(short, long)]
        input: PathBuf,

        /// Directory to store migrations
        #[arg(short, long, default_value = "migrations")]
        migrations_dir: PathBuf,

        /// Database URL for dialect detection (optional)
        #[arg(short, long, env = "DATABASE_URL")]
        database_url: Option<String>,
    },
    /// Apply pending migrations to the database
    Apply {
        /// Directory where migrations are stored
        #[arg(short, long, default_value = "migrations")]
        migrations_dir: PathBuf,

        /// Database URL
        #[arg(short, long, env = "DATABASE_URL")]
        database_url: String,
    },
}

impl MigrationArgs {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            MigrationCommand::Generate { name, input, migrations_dir, database_url } => {
                self.generate(name, input, migrations_dir, database_url.as_deref()).await
            }
            MigrationCommand::Apply { migrations_dir, database_url } => {
                self.apply(migrations_dir, database_url).await
            }
        }
    }

    async fn generate(&self, name: &str, input: &PathBuf, migrations_dir: &PathBuf, database_url: Option<&str>) -> Result<()> {
        let content = tokio::fs::read_to_string(input).await
            .map_err(|e| kql_types::KqlError::io(format!("Failed to read KQL file: {}", e)))?;

        // 1. Get New MIR
        let mut parser = Parser::new(&content);
        let ast = parser.parse()?;
        let mut lowerer = Lowerer::new();
        let hir_db = lowerer.lower_program(&ast)?;
        let mut mir_gen = MirLowerer::new(hir_db);
        let new_mir = mir_gen.lower()?;

        // 2. Get Old MIR from migration manager
        let manager = MigrationManager::new(migrations_dir);
        let old_mir = manager.get_latest_mir()
            .map_err(|e| kql_types::KqlError::cli(format!("Failed to read latest MIR: {}", e)))?
            .unwrap_or_default();

        // 3. Diff
        let engine = MigrationEngine::new(old_mir, new_mir.clone());
        let steps = engine.diff();

        if steps.is_empty() {
            println!("No changes detected. Migration not generated.");
            return Ok(());
        }

        // 4. Generate SQL
        let dialect = if let Some(url) = database_url {
            if url.starts_with("postgres") || url.starts_with("postgresql") {
                SqlDialect::Postgres
            } else if url.starts_with("mysql") || url.starts_with("mariadb") {
                SqlDialect::MySql
            } else {
                SqlDialect::Sqlite
            }
        } else {
            SqlDialect::Postgres // Default
        };

        let sql_gen = SqlGenerator::new(MirProgram::default(), dialect);
        let sqls = sql_gen.generate_migration_sql(steps);

        // 5. Save to file
        let path = manager.create_migration(name, &sqls, Some(&new_mir))
            .map_err(|e| kql_types::KqlError::cli(format!("Failed to create migration file: {}", e)))?;

        println!("✓ Migration generated: {:?}", path);
        Ok(())
    }

    async fn apply(&self, migrations_dir: &PathBuf, database_url: &str) -> Result<()> {
        let manager = MigrationManager::new(migrations_dir);
        let migrations = manager.list_migrations()
            .map_err(|e| kql_types::KqlError::cli(format!("Failed to list migrations: {}", e)))?;

        if migrations.is_empty() {
            println!("No migrations found in {:?}", migrations_dir);
            return Ok(());
        }

        // Connect to database
        sqlx::any::install_default_drivers();
        let mut conn = AnyConnection::connect(database_url).await
            .map_err(|e| kql_types::KqlError::database(format!("Failed to connect to DB: {}", e)))?;

        // TODO: Track applied migrations in a table. 
        // For now, just execute all of them (simple version).
        
        for path in migrations {
            let sql = tokio::fs::read_to_string(&path).await
                .map_err(|e| kql_types::KqlError::io(format!("Failed to read migration {:?}: {}", path, e)))?;
            
            println!("Applying migration: {:?}...", path.file_name().unwrap());
            
            // Execute each statement in the file
            for stmt in sql.split(';') {
                let stmt = stmt.trim();
                if !stmt.is_empty() {
                    sqlx::query(stmt)
                        .execute(&mut conn)
                        .await
                        .map_err(|e| kql_types::KqlError::database(format!("Failed to execute SQL: {}\nError: {}", stmt, e)))?;
                }
            }
        }

        println!("✓ All migrations applied successfully!");
        Ok(())
    }
}
