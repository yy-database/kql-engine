pub mod manager;
use crate::mir::{Table, Column, Index, ForeignKey, MirProgram};

#[derive(Debug, Clone)]
pub enum MigrationStep {
    CreateTable(Table),
    DropTable(Table),
    RenameTable { old_name: String, new_name: String },
    AddColumn { table_name: String, column: Column },
    DropColumn { table_name: String, column: Column },
    RenameColumn { table_name: String, old_name: String, new_name: String },
    AlterColumn { table_name: String, old_column: Column, new_column: Column },
    AddIndex { table_name: String, index: Index },
    DropIndex { table_name: String, index: Index },
    AddForeignKey { table_name: String, foreign_key: ForeignKey },
    DropForeignKey { table_name: String, foreign_key: ForeignKey },
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
        for (name, old_table) in &self.old_mir.tables {
            if !self.new_mir.tables.contains_key(name) {
                steps.push(MigrationStep::DropTable(old_table.clone()));
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
        for (&name, &old_col) in &old_cols {
            if !new_cols.contains_key(name) {
                steps.push(MigrationStep::DropColumn {
                    table_name: table_name.to_string(),
                    column: old_col.clone(),
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

        // Check indexes
        let old_indexes: std::collections::HashMap<_, _> = old_table.indexes.iter().map(|i| (&i.name, i)).collect();
        let new_indexes: std::collections::HashMap<_, _> = new_table.indexes.iter().map(|i| (&i.name, i)).collect();

        for (&name, &old_idx) in &old_indexes {
            if !new_indexes.contains_key(name) {
                steps.push(MigrationStep::DropIndex {
                    table_name: table_name.to_string(),
                    index: old_idx.clone(),
                });
            }
        }

        for (&name, &new_idx) in &new_indexes {
            if let Some(&old_idx) = old_indexes.get(name) {
                if old_idx != new_idx {
                    steps.push(MigrationStep::DropIndex {
                        table_name: table_name.to_string(),
                        index: old_idx.clone(),
                    });
                    steps.push(MigrationStep::AddIndex {
                        table_name: table_name.to_string(),
                        index: new_idx.clone(),
                    });
                }
            } else {
                steps.push(MigrationStep::AddIndex {
                    table_name: table_name.to_string(),
                    index: new_idx.clone(),
                });
            }
        }

        // Check foreign keys
        let old_fks: std::collections::HashMap<_, _> = old_table.foreign_keys.iter().map(|f| (&f.name, f)).collect();
        let new_fks: std::collections::HashMap<_, _> = new_table.foreign_keys.iter().map(|f| (&f.name, f)).collect();

        for (&name, &old_fk) in &old_fks {
            if !new_fks.contains_key(name) {
                steps.push(MigrationStep::DropForeignKey {
                    table_name: table_name.to_string(),
                    foreign_key: old_fk.clone(),
                });
            }
        }

        for (&name, &new_fk) in &new_fks {
            if let Some(&old_fk) = old_fks.get(name) {
                if old_fk != new_fk {
                    steps.push(MigrationStep::DropForeignKey {
                        table_name: table_name.to_string(),
                        foreign_key: old_fk.clone(),
                    });
                    steps.push(MigrationStep::AddForeignKey {
                        table_name: table_name.to_string(),
                        foreign_key: new_fk.clone(),
                    });
                }
            } else {
                steps.push(MigrationStep::AddForeignKey {
                    table_name: table_name.to_string(),
                    foreign_key: new_fk.clone(),
                });
            }
        }

        steps
    }
}
