use super::*;
use crate::hir::{HirDatabase, HirStruct, HirType, PrimitiveType};
use kql_types::Result;

pub struct MirLowerer {
    pub hir_db: HirDatabase,
    pub mir_db: MirDatabase,
}

impl MirLowerer {
    pub fn new(hir_db: HirDatabase) -> Self {
        Self {
            hir_db,
            mir_db: MirDatabase::default(),
        }
    }

    pub fn lower(&mut self) -> Result<MirDatabase> {
        for s in self.hir_db.structs.values() {
            let table = self.lower_struct_to_table(s)?;
            self.mir_db.tables.insert(table.name.clone(), table);
        }
        Ok(self.mir_db.clone())
    }

    fn lower_struct_to_table(&self, s: &HirStruct) -> Result<Table> {
        let mut columns = Vec::new();
        let mut primary_key = None;
        let indexes = Vec::new();

        for f in &s.fields {
            let mut is_pk = false;
            let mut is_nullable = false;

            // Check if type is Key<T> or Optional<T>
            let mut current_ty = &f.ty;
            loop {
                match current_ty {
                    HirType::Key(inner) => {
                        is_pk = true;
                        current_ty = inner;
                    }
                    HirType::Optional(inner) => {
                        is_nullable = true;
                        current_ty = inner;
                    }
                    _ => break,
                }
            }

            let mut col = Column {
                name: f.name.clone(),
                ty: self.lower_hir_type_to_mir(current_ty)?,
                nullable: is_nullable,
                auto_increment: false,
                default: None,
            };

            if is_pk && primary_key.is_none() {
                primary_key = Some(vec![f.name.clone()]);
            }

            for attr in &f.attrs {
                match attr.name.as_str() {
                    "primary_key" => {
                        if primary_key.is_none() {
                            primary_key = Some(vec![f.name.clone()]);
                        }
                    }
                    "auto_increment" => {
                        col.auto_increment = true;
                    }
                    "nullable" => {
                        col.nullable = true;
                    }
                    _ => {}
                }
            }
            columns.push(col);
        }

        // Also check struct-level attributes for composite primary keys or indexes
        for attr in &s.attrs {
            match attr.name.as_str() {
                "primary_key" => {
                    // TODO: Handle composite PK from args
                }
                "index" => {
                    // TODO: Handle index from args
                }
                _ => {}
            }
        }

        Ok(Table {
            name: s.name.clone(),
            columns,
            primary_key,
            indexes,
        })
    }

    fn lower_hir_type_to_mir(&self, ty: &HirType) -> Result<ColumnType> {
        match ty {
            HirType::Primitive(p) => match p {
                PrimitiveType::Integer32 => Ok(ColumnType::Int32),
                PrimitiveType::Float32 => Ok(ColumnType::Float32),
                PrimitiveType::String => Ok(ColumnType::String(None)),
                PrimitiveType::Bool => Ok(ColumnType::Bool),
                PrimitiveType::DateTime => Ok(ColumnType::DateTime),
                PrimitiveType::Uuid => Ok(ColumnType::Uuid),
            },
            HirType::Optional(inner) => self.lower_hir_type_to_mir(inner),
            HirType::Key(inner) => self.lower_hir_type_to_mir(inner),
            HirType::List(_inner) => Ok(ColumnType::Json), // Handle list as JSON for now
            _ => Ok(ColumnType::Json),
        }
    }
}
