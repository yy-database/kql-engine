use crate::hir::{
    HirExprKind, HirId, HirLiteral, HirProgram, HirType, PrimitiveType,
};
use crate::mir::*;
use kql_types::Result;
use std::collections::HashMap;

pub struct MirLowerer {
    hir_db: HirProgram,
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
    pub fn new(hir_db: HirProgram) -> Self {
        Self { hir_db }
    }

    pub fn lower(&mut self) -> Result<MirProgram> {
        let mut mir_db = MirProgram::default();
        let mut relations: HashMap<String, Vec<(HirId, String, HirType)>> = HashMap::new();

        // 1. Pre-scan for relations (especially Many-to-Many)
        for s in self.hir_db.structs.values() {
            for f in &s.fields {
                for attr in &f.attrs {
                    if attr.name == "relation" {
                        if let Some(name_arg) = attr.args.iter().find(|a| a.name.as_deref() == Some("name")) {
                            if let HirExprKind::Literal(HirLiteral::String(rel_name)) = &name_arg.value.kind {
                                relations.entry(rel_name.clone()).or_default().push((s.id, f.name.clone(), f.ty.clone()));
                            }
                        }
                    }
                }
            }
        }

        // 2. Lower structs to tables
        for s in self.hir_db.structs.values() {
            let mut table_name = to_snake_case(&s.name);
            let mut schema = s.schema.clone(); // Use schema from HIR (propagated from namespace)
            let mut primary_key = None;
            let mut indexes = Vec::new();
            let mut foreign_keys = Vec::new();

            for attr in &s.attrs {
                match attr.name.as_str() {
                    "table" => {
                        for arg in &attr.args {
                            if let Some(name) = &arg.name {
                                if name == "schema" {
                                    if let HirExprKind::Literal(HirLiteral::String(s)) = &arg.value.kind {
                                        schema = Some(s.clone()); // Override with explicit @table(schema: ...)
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
            let mut relations_list = Vec::new();
            for f in &s.fields {
                // Check if it's a virtual relation field (has @relation and refers to a struct)
                let mut is_virtual = false;
                match &f.ty {
                    HirType::Struct(_) | HirType::Optional(_) | HirType::List(_) => {
                        for attr in &f.attrs {
                            if attr.name == "relation" {
                                is_virtual = true;
                                break;
                            }
                        }
                    }
                    _ => {}
                }

                if is_virtual {
                    let mut foreign_key_column = None;
                    let mut target_table = None;
                    let mut target_column = Some("id".to_string());

                    for attr in &f.attrs {
                        if attr.name == "relation" {
                            for arg in &attr.args {
                                match arg.name.as_deref() {
                                    Some("foreign_key") => {
                                        if let HirExprKind::Literal(HirLiteral::String(col)) = &arg.value.kind {
                                            foreign_key_column = Some(col.clone());
                                        } else if let HirExprKind::Symbol(col) = &arg.value.kind {
                                            foreign_key_column = Some(col.clone());
                                        }
                                    }
                                    Some("target") => {
                                        if let HirExprKind::Literal(HirLiteral::String(t)) = &arg.value.kind {
                                            target_table = Some(t.clone());
                                        } else if let HirExprKind::Symbol(t) = &arg.value.kind {
                                            target_table = Some(t.clone());
                                        }
                                    }
                                    Some("references") => {
                                        if let HirExprKind::Literal(HirLiteral::String(col)) = &arg.value.kind {
                                            target_column = Some(col.clone());
                                        } else if let HirExprKind::Symbol(col) = &arg.value.kind {
                                            target_column = Some(col.clone());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    if let Some(fk_col) = foreign_key_column {
                        let target_table = target_table.unwrap_or_else(|| {
                            let mut current_ty = &f.ty;
                            loop {
                                match current_ty {
                                    HirType::Optional(inner) => current_ty = inner,
                                    HirType::List(inner) => current_ty = inner,
                                    _ => break,
                                }
                            }
                            if let HirType::Struct(id) = current_ty {
                                if let Some(target_struct) = self.hir_db.structs.get(id) {
                                    return to_snake_case(&target_struct.name);
                                }
                            }
                            "unknown".to_string()
                        });

                        relations_list.push(Relation {
                            name: f.name.clone(),
                            foreign_key_column: fk_col,
                            target_table,
                            target_column: target_column.unwrap_or_else(|| "id".to_string()),
                        });
                    }
                    continue;
                }

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
                                        HirLiteral::Null => "NULL".to_string(),
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

                // Check for foreign key relation
                let ref_id = if let HirType::Key { entity, inner, .. } = &f.ty {
                    entity.or_else(|| {
                        if let HirType::Struct(id) = inner.as_ref() {
                            Some(*id)
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };

                if let Some(ref_id) = ref_id {
                    if let Some(ref_struct) = self.hir_db.structs.get(&ref_id) {
                        let ref_table_name = to_snake_case(&ref_struct.name);
                        let ref_schema = ref_struct.schema.clone();
                        let mut referenced_columns = vec!["id".to_string()];
                        let mut on_delete = None;
                        let mut on_update = None;

                        for attr in &f.attrs {
                            if attr.name == "relation" {
                                for arg in &attr.args {
                                    match arg.name.as_deref() {
                                        Some("references") => {
                                            if let HirExprKind::Literal(HirLiteral::String(col)) = &arg.value.kind {
                                                referenced_columns = vec![col.clone()];
                                            } else if let HirExprKind::Symbol(col) = &arg.value.kind {
                                                referenced_columns = vec![col.clone()];
                                            }
                                        }
                                        Some("on_delete") => {
                                            if let HirExprKind::Symbol(action) = &arg.value.kind {
                                                on_delete = self.parse_reference_action(action);
                                            }
                                        }
                                        Some("on_update") => {
                                            if let HirExprKind::Symbol(action) = &arg.value.kind {
                                                on_update = self.parse_reference_action(action);
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        foreign_keys.push(ForeignKey {
                            name: format!("{}_{}_fk", table_name, f.name),
                            columns: vec![f.name.clone()],
                            referenced_schema: ref_schema,
                            referenced_table: ref_table_name,
                            referenced_columns,
                            on_delete,
                            on_update,
                        });
                    }
                }

                if let HirType::ForeignKey { name, entity } = &f.ty {
                    if let Some(target_struct) = self.hir_db.structs.get(entity) {
                        let target_table_name = to_snake_case(&target_struct.name);
                        let target_schema = target_struct.schema.clone();
                        let rel_name = name.clone().unwrap_or_else(|| target_table_name.clone());

                        relations_list.push(Relation {
                            name: rel_name,
                            foreign_key_column: f.name.clone(),
                            target_table: target_table_name.clone(),
                            target_column: "id".to_string(),
                        });

                        // Also add the actual foreign key constraint
                        let mut on_delete = None;
                        let mut on_update = None;
                        for attr in &f.attrs {
                            if attr.name == "relation" {
                                for arg in &attr.args {
                                    match arg.name.as_deref() {
                                        Some("on_delete") => {
                                            if let HirExprKind::Symbol(action) = &arg.value.kind {
                                                on_delete = self.parse_reference_action(action);
                                            }
                                        }
                                        Some("on_update") => {
                                            if let HirExprKind::Symbol(action) = &arg.value.kind {
                                                on_update = self.parse_reference_action(action);
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        foreign_keys.push(ForeignKey {
                            name: format!("{}_{}_fk", table_name, f.name),
                            columns: vec![f.name.clone()],
                            referenced_schema: target_schema,
                            referenced_table: target_table_name,
                            referenced_columns: vec!["id".to_string()],
                            on_delete,
                            on_update,
                        });
                    }
                }
            }

            let full_name = if let Some(ns) = &s.namespace {
                format!("{}::{}", ns, s.name)
            } else {
                s.name.clone()
            };

            mir_db.tables.insert(
                full_name,
                Table {
                    schema,
                    name: table_name,
                    columns,
                    primary_key,
                    indexes,
                    foreign_keys,
                    relations: relations_list,
                },
            );
        }

        // 3. Generate junction tables for Many-to-Many relations
        for (rel_name, fields) in relations {
            if fields.len() == 2 {
                let (id1, _f1, t1) = &fields[0];
                let (id2, _f2, t2) = &fields[1];

                if matches!(t1, HirType::List(_)) && matches!(t2, HirType::List(_)) {
                    let s1 = &self.hir_db.structs[id1];
                    let s2 = &self.hir_db.structs[id2];

                    let junction_table_name = to_snake_case(&rel_name);
                    let col1_name = format!("{}_id", to_snake_case(&s1.name));
                    let col2_name = format!("{}_id", to_snake_case(&s2.name));

                    let col1_ty = self.get_pk_type(id1)?;
                    let col2_ty = self.get_pk_type(id2)?;

                    let columns = vec![
                        Column {
                            name: col1_name.clone(),
                            ty: col1_ty,
                            nullable: false,
                            auto_increment: false,
                            default: None,
                        },
                        Column {
                            name: col2_name.clone(),
                            ty: col2_ty,
                            nullable: false,
                            auto_increment: false,
                            default: None,
                        },
                    ];

                    let foreign_keys = vec![
                        ForeignKey {
                            name: format!("{}_{}_fk", junction_table_name, col1_name),
                            columns: vec![col1_name.clone()],
                            referenced_schema: s1.schema.clone(),
                            referenced_table: to_snake_case(&s1.name),
                            referenced_columns: vec!["id".to_string()],
                            on_delete: Some(ReferenceAction::Cascade),
                            on_update: Some(ReferenceAction::Cascade),
                        },
                        ForeignKey {
                            name: format!("{}_{}_fk", junction_table_name, col2_name),
                            columns: vec![col2_name.clone()],
                            referenced_schema: s2.schema.clone(),
                            referenced_table: to_snake_case(&s2.name),
                            referenced_columns: vec!["id".to_string()],
                            on_delete: Some(ReferenceAction::Cascade),
                            on_update: Some(ReferenceAction::Cascade),
                        },
                    ];

                    mir_db.tables.insert(
                        rel_name.clone(),
                        Table {
                            schema: s1.schema.clone(),
                            name: junction_table_name,
                            columns,
                            primary_key: Some(vec![col1_name, col2_name]),
                            indexes: Vec::new(),
                            foreign_keys,
                            relations: Vec::new(),
                        },
                    );
                }
            }
        }

        Ok(mir_db)
    }

    fn get_pk_type(&self, id: &HirId) -> Result<ColumnType> {
        if let Some(s) = self.hir_db.structs.get(id) {
            for f in &s.fields {
                if f.name == "id" {
                    return self.lower_type(&f.ty);
                }
            }
        }
        Ok(ColumnType::I32)
    }

    fn parse_reference_action(&self, action: &str) -> Option<ReferenceAction> {
        match action.to_lowercase().as_str() {
            "no_action" => Some(ReferenceAction::NoAction),
            "restrict" => Some(ReferenceAction::Restrict),
            "cascade" => Some(ReferenceAction::Cascade),
            "set_null" => Some(ReferenceAction::SetNull),
            "set_default" => Some(ReferenceAction::SetDefault),
            _ => None,
        }
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
            HirType::ForeignKey { entity, .. } => self.get_pk_type(entity),
            HirType::Key { entity, inner } => {
                let actual_entity = entity.or_else(|| {
                    if let HirType::Struct(id) = inner.as_ref() {
                        Some(*id)
                    } else {
                        None
                    }
                });

                if let Some(entity_id) = actual_entity {
                    if let Some(s) = self.hir_db.structs.get(&entity_id) {
                        for f in &s.fields {
                            if f.name == "id" {
                                return self.lower_type(&f.ty);
                            }
                        }
                    }
                }
                self.lower_type(inner)
            }
            HirType::Null => Ok(ColumnType::Json),
            HirType::Unknown => Ok(ColumnType::Json),
        }
    }
}
