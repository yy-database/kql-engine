use std::path::{Path, PathBuf};
use std::fs;
use chrono::Local;
use anyhow::{Result, Context};

pub struct MigrationManager {
    pub migration_dir: PathBuf,
}

impl MigrationManager {
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            migration_dir: dir.as_ref().to_path_buf(),
        }
    }

    pub fn setup(&self) -> Result<()> {
        if !self.migration_dir.exists() {
            fs::create_dir_all(&self.migration_dir)
                .with_context(|| format!("Failed to create migration directory: {:?}", self.migration_dir))?;
        }
        Ok(())
    }

    pub fn create_migration(&self, name: &str, sql_statements: &[String], mir_state: Option<&crate::mir::MirProgram>) -> Result<PathBuf> {
        self.setup()?;

        let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
        let base_name = format!("{}_{}", timestamp, name.replace(" ", "_"));
        let sql_filename = format!("{}.sql", base_name);
        let sql_path = self.migration_dir.join(sql_filename);

        let content = sql_statements.join("\n");
        fs::write(&sql_path, content)
            .with_context(|| format!("Failed to write migration file: {:?}", sql_path))?;

        if let Some(mir) = mir_state {
            let mir_filename = format!("{}.mir.json", base_name);
            let mir_path = self.migration_dir.join(mir_filename);
            let mir_json = serde_json::to_string_pretty(mir)?;
            fs::write(&mir_path, mir_json)
                .with_context(|| format!("Failed to write MIR state: {:?}", mir_path))?;
        }

        Ok(sql_path)
    }

    pub fn get_latest_mir(&self) -> Result<Option<crate::mir::MirProgram>> {
        let migrations = self.list_migrations()?;
        if migrations.is_empty() {
            return Ok(None);
        }

        // Search backwards for the latest .mir.json file
        for path in migrations.iter().rev() {
            let mir_path = path.with_extension("mir.json");
            if mir_path.exists() {
                let content = fs::read_to_string(&mir_path)?;
                let mir: crate::mir::MirProgram = serde_json::from_str(&content)?;
                return Ok(Some(mir));
            }
        }

        Ok(None)
    }

    pub fn list_migrations(&self) -> Result<Vec<PathBuf>> {
        if !self.migration_dir.exists() {
            return Ok(vec![]);
        }

        let mut migrations = Vec::new();
        for entry in fs::read_dir(&self.migration_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "sql") {
                migrations.push(path);
            }
        }

        migrations.sort();
        Ok(migrations)
    }
}
