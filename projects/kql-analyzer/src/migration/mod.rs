use crate::mir::{Table, Column, Index, ForeignKey, MirProgram};

#[derive(Debug, Clone)]
pub enum MigrationStep {
    CreateTable(Table),
    DropTable(String),
    RenameTable { old_name: String, new_name: String },
    AddColumn { table_name: String, column: Column },
    DropColumn { table_name: String, column_name: String },
    RenameColumn { table_name: String, old_name: String, new_name: String },
    AlterColumn { table_name: String, old_column: Column, new_column: Column },
    AddIndex { table_name: String, index: Index },
    DropIndex { table_name: String, index_name: String },
    AddForeignKey { table_name: String, foreign_key: ForeignKey },
    DropForeignKey { table_name: String, foreign_key_name: String },
}

pub struct MigrationEngine {
    pub old_mir: MirProgram,
    pub new_mir: MirProgram,
}

impl MigrationEngine {
    pub fn new(old_mir: MirProgram, new_mir: MirProgram) -> Self {
        Self { old_mir, new_mir }
    }

    pub fn diff(&self) -> Vec<MigrationStep> {
        let mut steps = Vec::new();

        // 1. Detect dropped tables
        for (name, _) in &self.old_mir.tables {
            if !self.new_mir.tables.contains_key(name) {
                steps.push(MigrationStep::DropTable(name.clone()));
            }
        }

        // 2. Detect new or altered tables
        for (name, new_table) in &self.new_mir.tables {
            if let Some(old_table) = self.old_mir.tables.get(name) {
                // Table exists, check for changes
                steps.extend(self.diff_table(name, old_table, new_table));
            } else {
                // New table
                steps.push(MigrationStep::CreateTable(new_table.clone()));
            }
        }

        steps
    }

    fn diff_table(&self, table_name: &str, old_table: &Table, new_table: &Table) -> Vec<MigrationStep> {
        let mut steps = Vec::new();

        // Check columns
        let old_cols: std::collections::HashMap<_, _> = old_table.columns.iter().map(|c| (&c.name, c)).collect();
        let new_cols: std::collections::HashMap<_, _> = new_table.columns.iter().map(|c| (&c.name, c)).collect();

        // Dropped columns
        for (&name, &_) in &old_cols {
            if !new_cols.contains_key(name) {
                steps.push(MigrationStep::DropColumn {
                    table_name: table_name.to_string(),
                    column_name: name.clone(),
                });
            }
        }

        // New or altered columns
        for (&name, &new_col) in &new_cols {
            if let Some(&old_col) = old_cols.get(name) {
                if old_col != new_col {
                    steps.push(MigrationStep::AlterColumn {
                        table_name: table_name.to_string(),
                        old_column: old_col.clone(),
                        new_column: new_col.clone(),
                    });
                }
            } else {
                steps.push(MigrationStep::AddColumn {
                    table_name: table_name.to_string(),
                    column: new_col.clone(),
                });
            }
        }

        // TODO: Check indexes and foreign keys

        steps
    }
}
