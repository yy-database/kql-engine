use clap::Args;
use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::lir::sql_gen::SqlGenerator;
use sqlx::AnyConnection;
use sqlx::Connection;
use std::path::PathBuf;
use kql_types::Result;

#[derive(Args)]
pub struct MigrateArgs {
    /// The KQL file to migrate
    pub input: PathBuf,

    /// Database URL (e.g. sqlite:test.db, postgres://user:pass@localhost/db)
    #[arg(short, long, env = "DATABASE_URL")]
    pub database_url: String,

    /// Print SQL without executing
    #[arg(long)]
    pub dry_run: bool,
}

impl MigrateArgs {
    pub async fn run(&self) -> Result<()> {
        let content = tokio::fs::read_to_string(&self.input).await?;

        // 1. Parse
        let mut parser = Parser::new(&content);
        let ast = parser.parse()?;

        // 2. HIR Lowering
        let mut lowerer = Lowerer::new();
        let hir_db = lowerer.lower_program(&ast)?;

        // 3. MIR Lowering
        let mut mir_gen = MirLowerer::new(hir_db);
        let mir_db = mir_gen.lower()?;

        // 4. LIR (SQL) Generation
        let dialect = if self.database_url.starts_with("postgres") || self.database_url.starts_with("postgresql") {
            kql_analyzer::lir::SqlDialect::Postgres
        } else if self.database_url.starts_with("mysql") || self.database_url.starts_with("mariadb") {
            kql_analyzer::lir::SqlDialect::MySql
        } else {
            kql_analyzer::lir::SqlDialect::Sqlite
        };

        let sql_gen = SqlGenerator::new(mir_db, dialect);
        let sql_statements = sql_gen.generate_ddl_sql();

        if self.dry_run {
            println!("-- Dry run: SQL statements to be executed:");
            for sql in &sql_statements {
                println!("{};", sql);
            }
            return Ok(());
        }

        // 5. Execute via sqlx
        println!("Connecting to database: {}", self.database_url);
        
        // Use AnyConnection to support multiple databases based on URL scheme
        sqlx::any::install_default_drivers();
        let mut conn = AnyConnection::connect(&self.database_url).await?;

        for sql in sql_statements {
            println!("Executing: {}...", sql);
            sqlx::query(&sql)
                .execute(&mut conn)
                .await?;
        }

        println!("Migration completed successfully!");
        Ok(())
    }
}
