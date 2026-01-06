use crate::hir::{
    HirArgument, HirExpr, HirExprKind, HirId, HirLiteral, HirProgram, HirType, PrimitiveType,
};
use crate::mir::*;
use kql_types::Result;
use std::collections::HashMap;

pub struct MirLowerer {
    hir_db: HirProgram,
}

pub fn to_snake_case(s: &str) -> String {
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

    fn expr_to_string(&self, expr: &HirExpr) -> Option<String> {
        match &expr.kind {
            HirExprKind::Literal(HirLiteral::String(s)) => Some(s.clone()),
            HirExprKind::Symbol(s) => Some(s.clone()),
            HirExprKind::Member { member, .. } => Some(member.clone()),
            _ => None,
        }
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
            // Skip structs that are marked as @layout(json) - they don't get their own table
            if matches!(s.layout, Some(crate::hir::StructLayout::Json)) {
                continue;
            }

            let mut table_name = to_snake_case(&s.name);
            let mut schema = s.schema.clone(); // Use schema from HIR (propagated from namespace)
            let mut struct_primary_key = None;
            let mut field_primary_keys = Vec::new();
            let mut indexes = Vec::new();
            let mut foreign_keys = Vec::new();
            let mut lifecycle_hooks = Vec::new();
            let mut audit = false;
            let mut soft_delete = false;

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
                            struct_primary_key = Some(pk_cols);
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
                    "before_save" => {
                        if let Some(arg) = attr.args.get(0) {
                            if let HirExprKind::Symbol(func) = &arg.value.kind {
                                lifecycle_hooks.push(LifecycleHook {
                                    event: LifecycleEvent::BeforeSave,
                                    function: func.clone(),
                                });
                            }
                        }
                    }
                    "after_save" => {
                        if let Some(arg) = attr.args.get(0) {
                            if let HirExprKind::Symbol(func) = &arg.value.kind {
                                lifecycle_hooks.push(LifecycleHook {
                                    event: LifecycleEvent::AfterSave,
                                    function: func.clone(),
                                });
                            }
                        }
                    }
                    "before_delete" => {
                        if let Some(arg) = attr.args.get(0) {
                            if let HirExprKind::Symbol(func) = &arg.value.kind {
                                lifecycle_hooks.push(LifecycleHook {
                                    event: LifecycleEvent::BeforeDelete,
                                    function: func.clone(),
                                });
                            }
                        }
                    }
                    "after_delete" => {
                        if let Some(arg) = attr.args.get(0) {
                            if let HirExprKind::Symbol(func) = &arg.value.kind {
                                lifecycle_hooks.push(LifecycleHook {
                                    event: LifecycleEvent::AfterDelete,
                                    function: func.clone(),
                                });
                            }
                        }
                    }
                    "audit" => {
                        audit = true;
                    }
                    "soft_delete" => {
                        soft_delete = true;
                    }
                    _ => {}
                }
            }

            let mut columns = Vec::new();
            let mut relations_list = Vec::new();
            for f in &s.fields {
                if let HirType::Relation { name: rel_name, target, is_list, foreign_key, references, .. } = &f.ty {
                     // Virtual relation fields for @relation
                     if let Some(target_struct) = self.hir_db.structs.get(target) {
                         let foreign_key_column = foreign_key.clone();
                         let mut target_table = Some(to_snake_case(&target_struct.name));
                         let target_column = references.clone().or_else(|| Some("id".to_string()));

                         for attr in &f.attrs {
                             if attr.name == "relation" {
                                 for arg in &attr.args {
                                     match arg.name.as_deref() {
                                         Some("target") => {
                                             if let Some(s) = self.expr_to_string(&arg.value) {
                                                 target_table = Some(s);
                                             }
                                         }
                                         _ => {}
                                     }
                                 }
                             }
                         }

                         let fk_col = foreign_key_column.unwrap_or_else(|| {
                             // Try to find a matching foreign key in the current table
                             for f2 in &s.fields {
                                 if let HirType::ForeignKey { entity, .. } = &f2.ty {
                                     if entity == target {
                                         return f2.name.clone();
                                     }
                                 }
                                 if let HirType::Key { entity: Some(ent), .. } = &f2.ty {
                                     if ent == target {
                                         return f2.name.clone();
                                     }
                                 }
                             }
                             "id".to_string() // Fallback
                         });

                         relations_list.push(Relation {
                            name: f.name.clone(),
                            relation_name: rel_name.clone(),
                            foreign_key_column: fk_col,
                            target_table: target_table.unwrap_or_else(|| "unknown".to_string()),
                            target_column: target_column.unwrap_or_else(|| "id".to_string()),
                            is_list: *is_list,
                            fk_on_target: *is_list,
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
                            field_primary_keys.push(f.name.clone());
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
                                        HirLiteral::Star => "*".to_string(),
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
                                            if let Some(s) = self.expr_to_string(&arg.value) {
                                                referenced_columns = vec![s];
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
                            relation_name: None,
                            foreign_key_column: f.name.clone(),
                            target_table: target_table_name.clone(),
                            target_column: "id".to_string(),
                            is_list: false,
                            fk_on_target: false,
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

            // Add audit columns
            if audit {
                if !columns.iter().any(|c| c.name == "created_at") {
                    columns.push(Column {
                        name: "created_at".to_string(),
                        ty: ColumnType::DateTime,
                        nullable: false,
                        auto_increment: false,
                        default: Some("CURRENT_TIMESTAMP".to_string()),
                    });
                }
                if !columns.iter().any(|c| c.name == "updated_at") {
                    columns.push(Column {
                        name: "updated_at".to_string(),
                        ty: ColumnType::DateTime,
                        nullable: false,
                        auto_increment: false,
                        default: Some("CURRENT_TIMESTAMP".to_string()),
                    });
                }
            }

            // Add soft delete column
            if soft_delete {
                if !columns.iter().any(|c| c.name == "deleted_at") {
                    columns.push(Column {
                        name: "deleted_at".to_string(),
                        ty: ColumnType::DateTime,
                        nullable: true,
                        auto_increment: false,
                        default: None,
                    });
                }
            }

            // Add audit columns
            if audit {
                let audit_cols = [
                    ("created_at", ColumnType::DateTime, false, Some("CURRENT_TIMESTAMP".to_string())),
                    ("updated_at", ColumnType::DateTime, false, Some("CURRENT_TIMESTAMP".to_string())),
                    ("created_by", ColumnType::String(None), true, None),
                    ("updated_by", ColumnType::String(None), true, None),
                ];

                for (name, ty, nullable, default) in audit_cols {
                    if !columns.iter().any(|c| c.name == name) {
                        columns.push(Column {
                            name: name.to_string(),
                            ty,
                            nullable,
                            auto_increment: false,
                            default,
                        });
                    }
                }
            }

            // Determine primary key: struct-level takes precedence, then field-level
            let primary_key = if let Some(pk) = struct_primary_key {
                Some(pk)
            } else if !field_primary_keys.is_empty() {
                Some(field_primary_keys)
            } else {
                None
            };

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
                    lifecycle_hooks,
                    audit,
                    soft_delete,
                },
            );
        }

        // 3. Generate junction tables for Many-to-Many relations
        for (rel_name, fields) in relations {
            if fields.len() == 2 {
                let (id1, _f1, t1) = &fields[0];
                let (id2, _f2, t2) = &fields[1];

                let is_m2m = match (t1, t2) {
                    (HirType::Relation { is_list: true, .. }, HirType::Relation { is_list: true, .. }) => true,
                    (HirType::List(_), HirType::List(_)) => true, // Fallback for old style if any
                    _ => false,
                };

                if is_m2m {
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
                            lifecycle_hooks: Vec::new(),
                            audit: false,
                            soft_delete: false,
                        },
                    );
                }
            }
        }

        // 4. Lower queries from lets
        for let_decl in self.hir_db.lets.values() {
            if let Some(query) = self.lower_query(let_decl) {
                let full_name = if let Some(ns) = &let_decl.namespace {
                    format!("{}::{}", ns, let_decl.name)
                } else {
                    let_decl.name.clone()
                };
                mir_db.queries.insert(full_name, query);
            }
        }

        Ok(mir_db)
    }

    fn lower_query(&self, let_decl: &crate::hir::HirLet) -> Option<MirQuery> {
        let mut source_table = None;
        let mut joins = Vec::new();
        let mut selection = None;
        let mut projection = Vec::new();
        let mut current_alias = None;
        let mut current_table_name = None;

        self.process_query_expr(&let_decl.value, &mut source_table, &mut joins, &mut selection, &mut projection, &mut current_alias, &mut current_table_name);

        source_table.map(|table| MirQuery {
            name: let_decl.name.clone(),
            source_table: table,
            joins,
            selection,
            projection,
        })
    }

    fn process_query_expr(
        &self,
        expr: &HirExpr,
        source_table: &mut Option<String>,
        joins: &mut Vec<MirJoin>,
        selection: &mut Option<MirExpr>,
        projection: &mut Vec<MirProjection>,
        current_alias: &mut Option<String>,
        current_table_name: &mut Option<String>,
    ) {
        match &expr.kind {
            HirExprKind::Variable(id) => {
                if let Some(crate::hir::HirKind::Struct) = self.hir_db.id_to_kind.get(id) {
                    if let Some(s) = self.hir_db.structs.get(id) {
                        let table_name = to_snake_case(&s.name);
                        *source_table = Some(table_name.clone());
                        *current_alias = Some(table_name.clone());
                        *current_table_name = Some(table_name);
                        projection.push(MirProjection::All);
                    }
                }
            }
            HirExprKind::Symbol(name) => {
                // Check if this symbol is a table (struct)
                if let Some(id) = self.hir_db.name_to_id.get(name) {
                    if matches!(self.hir_db.id_to_kind.get(id), Some(crate::hir::HirKind::Struct)) {
                        let table_name = to_snake_case(name);
                        *source_table = Some(table_name.clone());
                        *current_alias = Some(table_name.clone());
                        *current_table_name = Some(table_name);
                        projection.push(MirProjection::All);
                    }
                }
            }
            HirExprKind::Member { object, member } => {
                self.process_query_expr(object, source_table, joins, selection, projection, current_alias, current_table_name);
                
                let struct_id = match &object.ty {
                    crate::hir::HirType::Struct(id) => Some(*id),
                    crate::hir::HirType::List(inner) => {
                        if let crate::hir::HirType::Struct(id) = inner.as_ref() {
                            Some(*id)
                        } else {
                            None
                        }
                    }
                    crate::hir::HirType::Relation { target, .. } => Some(*target),
                    _ => None,
                };

                // If the object has a relation with this member name, add a join
                if let Some(id) = struct_id {
                    if let Some(s) = self.hir_db.structs.get(&id) {
                        if let Some(field) = s.fields.iter().find(|f| &f.name == member) {
                            if let crate::hir::HirType::Relation { target, .. } = &field.ty {
                                if let Some(target_struct) = self.hir_db.structs.get(target) {
                                    let target_table = to_snake_case(&target_struct.name);
                                    let join_alias = member.clone();
                                    let source_alias = current_alias.clone().unwrap_or_else(|| "unknown".to_string());
                                    let source_table_name = current_table_name.clone().unwrap_or_else(|| "unknown".to_string());
                                    
                                    // Avoid duplicate joins with the same alias
                                    if !joins.iter().any(|j| &j.relation_name == &join_alias) {
                                        joins.push(MirJoin {
                                            source_table: source_table_name,
                                            source_alias,
                                            relation_name: join_alias.clone(),
                                            target_table: target_table.clone(),
                                            join_type: MirJoinType::Left,
                                            condition: None,
                                        });
                                    }
                                    
                                    *current_alias = Some(join_alias);
                                    *current_table_name = Some(target_table);
                                    
                                    // Change projection to target table if it was wildcard
                                    if projection.len() == 1 && matches!(projection[0], MirProjection::All) {
                                        projection.clear();
                                        projection.push(MirProjection::All);
                                    }
                                }
                            } else {
                                // Regular field access
                                projection.clear();
                                projection.push(MirProjection::Field(member.clone()));
                            }
                        }
                    }
                }
            }
            HirExprKind::Call { func, args } => {
                if let HirExprKind::Member { object, member } = &func.kind {
                    self.process_query_expr(object, source_table, joins, selection, projection, current_alias, current_table_name);
                    match member.as_str() {
                        "filter" | "where" => {
                            if let Some(arg) = args.get(0) {
                                let expr = match arg {
                                    HirArgument::Positional(e) => e,
                                    HirArgument::Named { value, .. } => value,
                                };
                                let cond = self.lower_expr_with_alias(expr, current_alias.as_deref());
                                *selection = match selection.take() {
                                    Some(existing) => Some(MirExpr::Binary {
                                        left: Box::new(existing),
                                        op: MirBinaryOp::And,
                                        right: Box::new(cond),
                                    }),
                                    None => Some(cond),
                                };
                            }
                        }
                        "select" => {
                            projection.clear();
                            for arg in args {
                                let expr = match arg {
                                    HirArgument::Positional(e) => e,
                                    HirArgument::Named { value, .. } => value,
                                };
                                projection.push(self.lower_projection_with_alias(expr, current_alias.as_deref()));
                            }
                        }
                        "count" | "sum" | "avg" | "max" | "min" => {
                            projection.clear();
                            projection.push(self.lower_projection_with_alias(expr, current_alias.as_deref()));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn lower_expr_with_alias(&self, expr: &HirExpr, alias: Option<&str>) -> MirExpr {
        match &expr.kind {
            HirExprKind::Literal(lit) => MirExpr::Literal(match lit {
                HirLiteral::Integer64(n) => MirLiteral::Integer64(*n),
                HirLiteral::Float64(f) => MirLiteral::Float64(*f),
                HirLiteral::String(s) => MirLiteral::String(s.clone()),
                HirLiteral::Bool(b) => MirLiteral::Bool(*b),
                HirLiteral::Null => MirLiteral::Null,
                HirLiteral::Star => MirLiteral::Star,
            }),
            HirExprKind::Binary { left, op, right } => MirExpr::Binary {
                left: Box::new(self.lower_expr_with_alias(left, alias)),
                op: self.lower_binary_op(*op),
                right: Box::new(self.lower_expr_with_alias(right, alias)),
            },
            HirExprKind::Unary { op, expr } => MirExpr::Unary {
                op: self.lower_unary_op(*op),
                expr: Box::new(self.lower_expr_with_alias(expr, alias)),
            },
            HirExprKind::Symbol(name) => MirExpr::Column {
                table_alias: alias.map(|s| s.to_string()),
                column: name.clone(),
            },
            HirExprKind::Member { object: _, member } => MirExpr::Column {
                table_alias: alias.map(|s| s.to_string()),
                column: member.clone(),
            },
            HirExprKind::Call { func, args } => {
                let func_name = if let HirExprKind::Symbol(name) = &func.kind {
                    Some(name.clone())
                } else if let HirExprKind::Member { member, .. } = &func.kind {
                    Some(member.clone())
                } else {
                    None
                };

                if let Some(name) = func_name {
                    MirExpr::Call {
                        func: name,
                        args: args.iter().map(|a| match a {
                            HirArgument::Positional(e) => self.lower_expr_with_alias(e, alias),
                            HirArgument::Named { value, .. } => self.lower_expr_with_alias(value, alias),
                        }).collect(),
                    }
                } else {
                    MirExpr::Literal(MirLiteral::Null)
                }
            }
            HirExprKind::Window(w) => {
                MirExpr::Window(MirWindow {
                    expr: Box::new(self.lower_expr_with_alias(&w.expr, alias)),
                    partition_by: w.partition_by.iter().map(|e| self.lower_expr_with_alias(e, alias)).collect(),
                    order_by: w.order_by.iter().map(|o| MirOrderBy {
                        expr: Box::new(self.lower_expr_with_alias(&o.expr, alias)),
                        desc: o.desc,
                    }).collect(),
                    alias: None,
                })
            }
            _ => MirExpr::Literal(MirLiteral::Null),
        }
    }

    fn lower_projection_with_alias(&self, expr: &HirExpr, alias: Option<&str>) -> MirProjection {
        match &expr.kind {
            HirExprKind::Symbol(name) => MirProjection::Field(name.clone()),
            HirExprKind::Member { member, .. } => MirProjection::Field(member.clone()),
            HirExprKind::Call { func, args } => {
                let func_name = if let HirExprKind::Symbol(name) = &func.kind {
                    Some(name.clone())
                } else if let HirExprKind::Member { member, .. } = &func.kind {
                    Some(member.clone())
                } else {
                    None
                };

                if let Some(name) = func_name {
                    MirProjection::Aggregation(MirAggregation {
                        func: name,
                        arg: Box::new(if let Some(arg) = args.get(0) {
                            match arg {
                                HirArgument::Positional(e) => self.lower_expr_with_alias(e, alias),
                                HirArgument::Named { value, .. } => self.lower_expr_with_alias(value, alias),
                            }
                        } else {
                            MirExpr::Literal(MirLiteral::Null)
                        }),
                        alias: None,
                    })
                } else {
                    MirProjection::Field("unknown".to_string())
                }
            }
            HirExprKind::Window(w) => {
                MirProjection::Window(MirWindow {
                    expr: Box::new(self.lower_expr_with_alias(&w.expr, alias)),
                    partition_by: w.partition_by.iter().map(|e| self.lower_expr_with_alias(e, alias)).collect(),
                    order_by: w.order_by.iter().map(|o| MirOrderBy {
                        expr: Box::new(self.lower_expr_with_alias(&o.expr, alias)),
                        desc: o.desc,
                    }).collect(),
                    alias: None,
                })
            }
            _ => MirProjection::Field("unknown".to_string()),
        }
    }

    fn lower_binary_op(&self, op: crate::hir::HirBinaryOp) -> MirBinaryOp {
        match op {
            crate::hir::HirBinaryOp::Add => MirBinaryOp::Add,
            crate::hir::HirBinaryOp::Sub => MirBinaryOp::Sub,
            crate::hir::HirBinaryOp::Mul => MirBinaryOp::Mul,
            crate::hir::HirBinaryOp::Div => MirBinaryOp::Div,
            crate::hir::HirBinaryOp::Mod => MirBinaryOp::Mod,
            crate::hir::HirBinaryOp::Eq => MirBinaryOp::Eq,
            crate::hir::HirBinaryOp::NotEq => MirBinaryOp::NotEq,
            crate::hir::HirBinaryOp::Gt => MirBinaryOp::Gt,
            crate::hir::HirBinaryOp::Lt => MirBinaryOp::Lt,
            crate::hir::HirBinaryOp::GtEq => MirBinaryOp::GtEq,
            crate::hir::HirBinaryOp::LtEq => MirBinaryOp::LtEq,
            crate::hir::HirBinaryOp::And => MirBinaryOp::And,
            crate::hir::HirBinaryOp::Or => MirBinaryOp::Or,
        }
    }

    fn lower_unary_op(&self, op: crate::hir::HirUnaryOp) -> MirUnaryOp {
        match op {
            crate::hir::HirUnaryOp::Neg => MirUnaryOp::Neg,
            crate::hir::HirUnaryOp::Not => MirUnaryOp::Not,
        }
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
                PrimitiveType::I8 => Ok(ColumnType::I8),
                PrimitiveType::I16 => Ok(ColumnType::I16),
                PrimitiveType::I32 => Ok(ColumnType::I32),
                PrimitiveType::I64 => Ok(ColumnType::I64),
                PrimitiveType::U8 => Ok(ColumnType::U8),
                PrimitiveType::U16 => Ok(ColumnType::U16),
                PrimitiveType::U32 => Ok(ColumnType::U32),
                PrimitiveType::U64 => Ok(ColumnType::U64),
                PrimitiveType::F32 => Ok(ColumnType::F32),
                PrimitiveType::F64 => Ok(ColumnType::F64),
                PrimitiveType::String => Ok(ColumnType::String(None)),
                PrimitiveType::Bool => Ok(ColumnType::Bool),
                PrimitiveType::DateTime => Ok(ColumnType::DateTime),
                PrimitiveType::Date => Ok(ColumnType::Date),
                PrimitiveType::Time => Ok(ColumnType::Time),
                PrimitiveType::Uuid => Ok(ColumnType::Uuid),
                PrimitiveType::D64 => Ok(ColumnType::Decimal64),
                PrimitiveType::D128 => Ok(ColumnType::Decimal128),
                PrimitiveType::Bytes => Ok(ColumnType::Bytes),
                PrimitiveType::Json => Ok(ColumnType::Json),
            },
            HirType::Struct(_) => Ok(ColumnType::Json),
            HirType::Enum(id) => {
                if let Some(e) = self.hir_db.enums.get(id) {
                    // If any variant has fields, it must be JSON
                    if e.variants.iter().any(|v| v.fields.is_some()) {
                        return Ok(ColumnType::Json);
                    }
                    if let Some(layout) = &e.layout {
                        return self.lower_type(layout);
                    }
                }
                Ok(ColumnType::I32)
            }
            HirType::List(_) => Ok(ColumnType::Json),
            HirType::Optional(inner) => self.lower_type(inner),
            HirType::ForeignKey { entity, .. } => self.get_pk_type(entity),
            HirType::Relation { .. } => Ok(ColumnType::Json), // Should be filtered out in higher level
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
