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
    /// Rollback the last applied migration
    Rollback {
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
            MigrationCommand::Rollback { migrations_dir, database_url } => {
                self.rollback(migrations_dir, database_url).await
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
        let up_sqls = sql_gen.generate_migration_sql(steps.clone());
        
        let down_steps: Vec<_> = steps.into_iter().rev().map(|s| s.invert()).collect();
        let down_sqls = sql_gen.generate_migration_sql(down_steps);

        // 5. Save to file
        let path = manager.create_migration(name, &up_sqls, &down_sqls, Some(&new_mir))
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

        // 1. Ensure kql_migrations table exists
        let create_table_sql = if database_url.starts_with("postgres") || database_url.starts_with("postgresql") {
            "CREATE TABLE IF NOT EXISTS kql_migrations (id SERIAL PRIMARY KEY, name TEXT NOT NULL UNIQUE, applied_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP)"
        } else if database_url.starts_with("mysql") || database_url.starts_with("mariadb") {
            "CREATE TABLE IF NOT EXISTS kql_migrations (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255) NOT NULL UNIQUE, applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)"
        } else {
            "CREATE TABLE IF NOT EXISTS kql_migrations (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL UNIQUE, applied_at DATETIME DEFAULT CURRENT_TIMESTAMP)"
        };

        sqlx::query(create_table_sql)
            .execute(&mut conn)
            .await
            .map_err(|e| kql_types::KqlError::database(format!("Failed to create migration tracking table: {}", e)))?;

        // 2. Get applied migrations
        let applied_migrations: Vec<String> = sqlx::query_as::<_, (String,)>("SELECT name FROM kql_migrations")
            .fetch_all(&mut conn)
            .await
            .map_err(|e| kql_types::KqlError::database(format!("Failed to fetch applied migrations: {}", e)))?
            .into_iter()
            .map(|r| r.0)
            .collect();
        
        let mut count = 0;
        for path in migrations {
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            
            if applied_migrations.contains(&filename) {
                continue;
            }

            let sql = tokio::fs::read_to_string(&path).await
                .map_err(|e| kql_types::KqlError::io(format!("Failed to read migration {:?}: {}", path, e)))?;
            
            println!("Applying migration: {}...", filename);
            
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

            // Record migration
            sqlx::query("INSERT INTO kql_migrations (name) VALUES (?)")
                .bind(&filename)
                .execute(&mut conn)
                .await
                .map_err(|e| kql_types::KqlError::database(format!("Failed to record migration {}: {}", filename, e)))?;
            
            count += 1;
        }

        if count == 0 {
            println!("No new migrations to apply.");
        } else {
            println!("✓ {} migration(s) applied successfully!", count);
        }
        Ok(())
    }

    async fn rollback(&self, migrations_dir: &PathBuf, database_url: &str) -> Result<()> {
        // Connect to database
        sqlx::any::install_default_drivers();
        let mut conn = AnyConnection::connect(database_url).await
            .map_err(|e| kql_types::KqlError::database(format!("Failed to connect to DB: {}", e)))?;

        // 1. Get the last applied migration
        let last_migration: Option<(String,)> = sqlx::query_as("SELECT name FROM kql_migrations ORDER BY applied_at DESC, id DESC LIMIT 1")
            .fetch_optional(&mut conn)
            .await
            .map_err(|e| kql_types::KqlError::database(format!("Failed to fetch last migration: {}", e)))?;

        let filename = match last_migration {
            Some((name,)) => name,
            None => {
                println!("No migrations have been applied yet.");
                return Ok(());
            }
        };

        // 2. Find the corresponding .down.sql file
        let down_filename = filename.replace(".up.sql", ".down.sql");
        let down_path = migrations_dir.join(&down_filename);

        if !down_path.exists() {
            return Err(kql_types::KqlError::cli(format!("Down migration file not found: {:?}", down_path)));
        }

        let sql = tokio::fs::read_to_string(&down_path).await
            .map_err(|e| kql_types::KqlError::io(format!("Failed to read down migration {:?}: {}", down_path, e)))?;

        println!("Rolling back migration: {}...", filename);

        // 3. Execute down SQL
        for stmt in sql.split(';') {
            let stmt = stmt.trim();
            if !stmt.is_empty() {
                sqlx::query(stmt)
                    .execute(&mut conn)
                    .await
                    .map_err(|e| kql_types::KqlError::database(format!("Failed to execute SQL: {}\nError: {}", stmt, e)))?;
            }
        }

        // 4. Remove from kql_migrations
        sqlx::query("DELETE FROM kql_migrations WHERE name = ?")
            .bind(&filename)
            .execute(&mut conn)
            .await
            .map_err(|e| kql_types::KqlError::database(format!("Failed to remove migration record: {}", e)))?;

        println!("✓ Migration rolled back successfully!");
        Ok(())
    }
}
