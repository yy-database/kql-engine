use crate::hir::{
    HirDatabase, HirExprKind, HirLiteral, HirType, PrimitiveType,
};
use crate::mir::*;
use kql_types::Result;

pub struct MirLowerer {
    hir_db: HirDatabase,
}

fn to_snake_case(s: &str) -> String {
    let mut snake = String::new();
    for (i, ch) in s.char_indices() {
        if i > 0 && ch.is_uppercase() {
            snake.push('_');
        }
        snake.push(ch.to_lowercase().next().unwrap());
    }
    snake
}

impl MirLowerer {
    pub fn new(hir_db: HirDatabase) -> Self {
        Self { hir_db }
    }

    pub fn lower(&mut self) -> Result<MirDatabase> {
        let mut mir_db = MirDatabase::default();

        for s in self.hir_db.structs.values() {
            let mut table_name = to_snake_case(&s.name);
            let mut schema = None;
            let mut primary_key = None;
            let mut indexes = Vec::new();

            for attr in &s.attrs {
                match attr.name.as_str() {
                    "table" => {
                        for arg in &attr.args {
                            if let Some(name) = &arg.name {
                                if name == "schema" {
                                    if let HirExprKind::Literal(HirLiteral::String(s)) = &arg.value.kind {
                                        schema = Some(s.clone());
                                    }
                                }
                            } else if let HirExprKind::Literal(HirLiteral::String(name)) = &arg.value.kind {
                                table_name = name.clone();
                            }
                        }
                    }
                    "primary_key" => {
                        let mut pk_cols = Vec::new();
                        for arg in &attr.args {
                            match &arg.value.kind {
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
                            match &arg.value.kind {
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
                        let mut idx_cols = Vec::new();
                        for arg in &attr.args {
                            match &arg.value.kind {
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

            let mut columns = Vec::new();
            for f in &s.fields {
                let (column_type, is_optional) = self.lower_type_with_nullability(&f.ty)?;
                let mut nullable = is_optional;
                let mut auto_increment = false;
                let mut default = None;

                for attr in &f.attrs {
                    match attr.name.as_str() {
                        "primary_key" => {
                            primary_key = Some(vec![f.name.clone()]);
                            nullable = false;
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
                        "auto_increment" => {
                            auto_increment = true;
                        }
                        "not_null" => {
                            nullable = false;
                        }
                        "nullable" => {
                            nullable = true;
                        }
                        "default" => {
                            if let Some(arg) = attr.args.get(0) {
                                if let HirExprKind::Literal(lit) = &arg.value.kind {
                                    default = Some(match lit {
                                        HirLiteral::Integer64(n) => n.to_string(),
                                        HirLiteral::Float64(f) => f.to_string(),
                                        HirLiteral::String(s) => format!("'{}'", s),
                                        HirLiteral::Bool(b) => b.to_string(),
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }

                columns.push(Column {
                    name: f.name.clone(),
                    ty: column_type,
                    nullable,
                    auto_increment,
                    default,
                });
            }

            mir_db.tables.insert(
                s.name.clone(),
                Table {
                    schema,
                    name: table_name,
                    columns,
                    primary_key,
                    indexes,
                },
            );
        }

        Ok(mir_db)
    }

    fn lower_type_with_nullability(&self, ty: &HirType) -> Result<(ColumnType, bool)> {
        match ty {
            HirType::Optional(inner) => {
                let (ty, _) = self.lower_type_with_nullability(inner)?;
                Ok((ty, true))
            }
            _ => {
                let ty = self.lower_type(ty)?;
                Ok((ty, false))
            }
        }
    }

    fn lower_type(&self, ty: &HirType) -> Result<ColumnType> {
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
            HirType::Optional(inner) => self.lower_type(inner),
            HirType::Key { inner, .. } => self.lower_type(inner),
            HirType::Unknown => Ok(ColumnType::Json),
        }
    }
}
