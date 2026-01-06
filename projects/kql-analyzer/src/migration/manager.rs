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

    pub fn create_migration(&self, name: &str, up_sql: &[String], down_sql: &[String], mir_state: Option<&crate::mir::MirProgram>) -> Result<PathBuf> {
        self.setup()?;

        let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
        let base_name = format!("{}_{}", timestamp, name.replace(" ", "_"));
        
        // Up migration
        let up_filename = format!("{}.up.sql", base_name);
        let up_path = self.migration_dir.join(up_filename);
        let up_content = up_sql.join("\n");
        fs::write(&up_path, up_content)
            .with_context(|| format!("Failed to write UP migration file: {:?}", up_path))?;

        // Down migration
        let down_filename = format!("{}.down.sql", base_name);
        let down_path = self.migration_dir.join(down_filename);
        let down_content = down_sql.join("\n");
        fs::write(&down_path, down_content)
            .with_context(|| format!("Failed to write DOWN migration file: {:?}", down_path))?;

        if let Some(mir) = mir_state {
            let mir_filename = format!("{}.mir.json", base_name);
            let mir_path = self.migration_dir.join(mir_filename);
            let mir_json = serde_json::to_string_pretty(mir)?;
            fs::write(&mir_path, mir_json)
                .with_context(|| format!("Failed to write MIR state: {:?}", mir_path))?;
        }

        Ok(up_path)
    }

    pub fn get_latest_mir(&self) -> Result<Option<crate::mir::MirProgram>> {
        let migrations = self.list_migrations()?;
        if migrations.is_empty() {
            return Ok(None);
        }

        // Search backwards for the latest .mir.json file
        for path in migrations.iter().rev() {
            let mir_path = path.to_str()
                .and_then(|s| s.strip_suffix(".up.sql"))
                .map(|s| PathBuf::from(format!("{}.mir.json", s)))
                .unwrap_or_else(|| path.with_extension("mir.json"));

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
            if path.is_file() && path.to_str().map_or(false, |s| s.ends_with(".up.sql")) {
                migrations.push(path);
            }
        }

        migrations.sort();
        Ok(migrations)
    }
}
