use super::*;
use crate::hir::{HirDatabase, HirStruct, HirType, PrimitiveType, HirExprKind, HirLiteral};
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
        let mut indexes = Vec::new();
        let mut table_name = s.name.clone();

        for f in &s.fields {
            let mut is_pk = false;
            let mut is_nullable = false;

            // Check if type is Key<T> or Optional<T>
            let mut current_ty = &f.ty;
            loop {
                match current_ty {
                    HirType::Key { inner, .. } => {
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
                    "unique" => {
                        indexes.push(Index {
                            name: format!("{}_{}_unique", table_name, f.name),
                            columns: vec![f.name.clone()],
                            unique: true,
                        });
                    }
                    "index" => {
                        indexes.push(Index {
                            name: format!("{}_{}_idx", table_name, f.name),
                            columns: vec![f.name.clone()],
                            unique: false,
                        });
                    }
                    _ => {}
                }
            }
            columns.push(col);
        }

        // Also check struct-level attributes for composite primary keys or indexes
        for attr in &s.attrs {
            match attr.name.as_str() {
                "table" => {
                    if let Some(arg) = attr.args.first() {
                        if let HirExprKind::Literal(HirLiteral::String(name)) = &arg.kind {
                            table_name = name.clone();
                        }
                    }
                }
                "primary_key" => {
                    let mut pk_cols = Vec::new();
                    for arg in &attr.args {
                        match &arg.kind {
                            HirExprKind::Symbol(name) => {
                                pk_cols.push(name.clone());
                            }
                            HirExprKind::Literal(HirLiteral::String(name)) => {
                                pk_cols.push(name.clone());
                            }
                            _ => {}
                        }
                    }
                    if !pk_cols.is_empty() {
                        primary_key = Some(pk_cols);
                    }
                }
                "index" => {
                    let mut idx_cols = Vec::new();
                    let idx_name = None;
                    let unique = false;

                    for arg in &attr.args {
                        match &arg.kind {
                            HirExprKind::Symbol(name) => {
                                idx_cols.push(name.clone());
                            }
                            HirExprKind::Literal(HirLiteral::String(name)) => {
                                // If it's the first arg and there are more, it might be the name
                                // But usually we expect symbols for columns.
                                // Let's keep it simple: all strings/symbols are columns for now.
                                idx_cols.push(name.clone());
                            }
                            _ => {}
                        }
                    }

                    // Check for named arguments or special flags if we had them.
                    // For now, if there's a @unique attribute on the same struct, it's different.
                    // But maybe @index(name: "foo", columns: [a, b], unique: true)
                    // Our current parser doesn't support named args in attributes easily.

                    if !idx_cols.is_empty() {
                        let name = idx_name.unwrap_or_else(|| {
                            format!("{}_{}_idx", table_name, idx_cols.join("_"))
                        });
                        indexes.push(Index {
                            name,
                            columns: idx_cols,
                            unique,
                        });
                    }
                }
                "unique" => {
                    // Similar to index but unique
                    let mut idx_cols = Vec::new();
                    for arg in &attr.args {
                        match &arg.kind {
                            HirExprKind::Symbol(name) | HirExprKind::Literal(HirLiteral::String(name)) => {
                                idx_cols.push(name.clone());
                            }
                            _ => {}
                        }
                    }
                    if !idx_cols.is_empty() {
                        let name = format!("{}_{}_unique", table_name, idx_cols.join("_"));
                        indexes.push(Index {
                            name,
                            columns: idx_cols,
                            unique: true,
                        });
                    }
                }
                _ => {}
            }
        }

        Ok(Table {
            name: table_name,
            columns,
            primary_key,
            indexes,
        })
    }

    fn lower_hir_type_to_mir(&self, ty: &HirType) -> Result<ColumnType> {
        match ty {
            HirType::Primitive(p) => match p {
                PrimitiveType::I32 => Ok(ColumnType::I32),
                PrimitiveType::I64 => Ok(ColumnType::I64),
                PrimitiveType::F32 => Ok(ColumnType::F32),
                PrimitiveType::F64 => Ok(ColumnType::F64),
                PrimitiveType::String => Ok(ColumnType::String(None)),
                PrimitiveType::Bool => Ok(ColumnType::Bool),
                PrimitiveType::DateTime => Ok(ColumnType::DateTime),
                PrimitiveType::Uuid => Ok(ColumnType::Uuid),
                PrimitiveType::D128 => Ok(ColumnType::Decimal128),
            },
            HirType::Struct(_) => Ok(ColumnType::Json),
            HirType::Enum(_) => Ok(ColumnType::I32),
            HirType::List(_) => Ok(ColumnType::Json),
            _ => Ok(ColumnType::String(None)),
        }
    }
}
