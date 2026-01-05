use super::*;
use kql_ast as ast;
use kql_types::{KqlError, Result, Span};

pub struct Lowerer {
    pub db: HirProgram,
    pub errors: Vec<KqlError>,
    pub current_struct_id: Option<HirId>,
    pub resolving_aliases: Vec<HirId>,
    pub ast_decls: std::collections::HashMap<HirId, ast::Decl>,
}

impl Lowerer {
    pub fn new() -> Self {
        Self { 
            db: HirProgram::default(), 
            errors: Vec::new(), 
            current_struct_id: None,
            resolving_aliases: Vec::new(),
            ast_decls: std::collections::HashMap::new(),
        }
    }

    pub fn lower_program(&mut self, ast_db: &ast::Database) -> Result<HirProgram> {
        self.lower_decls(ast_db.decls.clone())?;
        if !self.errors.is_empty() {
            return Err(self.errors[0].clone());
        }
        Ok(self.db.clone())
    }

    pub fn lower_decls(&mut self, decls: Vec<ast::Decl>) -> Result<()> {
        self.collect_names(decls.clone(), None, None)?;
        self.lower_content(decls, None, None)
    }

    fn collect_names(&mut self, decls: Vec<ast::Decl>, mut namespace: Option<String>, mut db_schema: Option<String>) -> Result<()> {
        let mut has_toplevel_ns = false;
        for decl in &decls {
            match decl {
                ast::Decl::Struct(s) => {
                    let full_name = if let Some(ns) = &namespace {
                        format!("{}::{}", ns, s.name.name)
                    } else {
                        s.name.name.clone()
                    };
                    let id = self.db.alloc_id();
                    self.db.name_to_id.insert(full_name, id);
                    self.db.id_to_kind.insert(id, HirKind::Struct);
                    self.ast_decls.insert(id, ast::Decl::Struct(s.clone()));
                }
                ast::Decl::Enum(e) => {
                    let full_name = if let Some(ns) = &namespace {
                        format!("{}::{}", ns, e.name.name)
                    } else {
                        e.name.name.clone()
                    };
                    let id = self.db.alloc_id();
                    self.db.name_to_id.insert(full_name, id);
                    self.db.id_to_kind.insert(id, HirKind::Enum);
                    self.ast_decls.insert(id, ast::Decl::Enum(e.clone()));
                }
                ast::Decl::Let(l) => {
                    let full_name = if let Some(ns) = &namespace {
                        format!("{}::{}", ns, l.name.name)
                    } else {
                        l.name.name.clone()
                    };
                    let id = self.db.alloc_id();
                    self.db.name_to_id.insert(full_name, id);
                    self.db.id_to_kind.insert(id, HirKind::Let);
                    self.ast_decls.insert(id, ast::Decl::Let(l.clone()));
                }
                ast::Decl::TypeAlias(t) => {
                    let full_name = if let Some(ns) = &namespace {
                        format!("{}::{}", ns, t.name.name)
                    } else {
                        t.name.name.clone()
                    };
                    let id = self.db.alloc_id();
                    self.db.name_to_id.insert(full_name, id);
                    self.db.id_to_kind.insert(id, HirKind::TypeAlias);
                    self.ast_decls.insert(id, ast::Decl::TypeAlias(t.clone()));
                }
                ast::Decl::Namespace(d) => {
                    if !d.is_block {
                        if has_toplevel_ns {
                            self.errors.push(KqlError::lint(d.span, "Only one top-level namespace is allowed in a single scope."));
                        }
                        has_toplevel_ns = true;
                        
                        if namespace.is_some() {
                             self.errors.push(KqlError::lint(d.span, "Top-level namespace cannot be nested within another namespace. Use block-style 'namespace { ... }' instead."));
                        }
                        
                        let (new_ns, new_schema) = self.get_ns_and_schema(d, &namespace, &db_schema);
                        namespace = Some(new_ns);
                        db_schema = new_schema;
                    } else {
                        let (sub_ns, sub_schema) = self.get_ns_and_schema(d, &namespace, &db_schema);
                        self.collect_names(d.decls.clone(), Some(sub_ns), sub_schema)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn get_ns_and_schema(&self, d: &ast::NamespaceDecl, current_ns: &Option<String>, current_schema: &Option<String>) -> (String, Option<String>) {
        let sub_namespace = if let Some(ns) = current_ns {
            format!("{}::{}", ns, d.name.name)
        } else {
            d.name.name.clone()
        };

        let mut sub_db_schema = current_schema.clone();
        for attr in &d.attrs {
            if attr.name.name == "schema" {
                if let Some(args) = &attr.args {
                    if !args.is_empty() {
                        if let ast::Expr::Literal(ast::LiteralExpr { kind: ast::LiteralKind::String(s), .. }) = &args[0].value {
                            sub_db_schema = Some(s.clone());
                        }
                    } else {
                        sub_db_schema = Some(d.name.name.clone());
                    }
                } else {
                    sub_db_schema = Some(d.name.name.clone());
                }
            }
        }
        (sub_namespace, sub_db_schema)
    }

    fn lower_content(&mut self, decls: Vec<ast::Decl>, mut namespace: Option<String>, mut db_schema: Option<String>) -> Result<()> {
        for decl in decls {
            match decl {
                ast::Decl::Struct(s) => {
                    let full_name = if let Some(ns) = &namespace {
                        format!("{}::{}", ns, s.name.name)
                    } else {
                        s.name.name.clone()
                    };
                    match self.lower_struct(s, namespace.clone(), db_schema.clone(), &full_name) {
                        Ok(hir_s) => { self.db.structs.insert(hir_s.id, hir_s); }
                        Err(e) => self.errors.push(e),
                    }
                }
                ast::Decl::Enum(e) => {
                    let full_name = if let Some(ns) = &namespace {
                        format!("{}::{}", ns, e.name.name)
                    } else {
                        e.name.name.clone()
                    };
                    match self.lower_enum(e, namespace.clone(), db_schema.clone(), &full_name) {
                        Ok(hir_e) => { self.db.enums.insert(hir_e.id, hir_e); }
                        Err(e) => self.errors.push(e),
                    }
                }
                ast::Decl::Let(l) => {
                    let full_name = if let Some(ns) = &namespace {
                        format!("{}::{}", ns, l.name.name)
                    } else {
                        l.name.name.clone()
                    };
                    match self.lower_let(l, namespace.clone(), &full_name) {
                        Ok(hir_l) => { self.db.lets.insert(hir_l.id, hir_l); }
                        Err(e) => self.errors.push(e),
                    }
                }
                ast::Decl::TypeAlias(t) => {
                    let full_name = if let Some(ns) = &namespace {
                        format!("{}::{}", ns, t.name.name)
                    } else {
                        t.name.name.clone()
                    };
                    match self.lower_type_alias(t, namespace.clone(), &full_name) {
                        Ok(hir_t) => { self.db.type_aliases.insert(hir_t.id, hir_t); }
                        Err(e) => self.errors.push(e),
                    }
                }
                ast::Decl::Namespace(d) => {
                    if !d.is_block {
                        let (new_ns, new_schema) = self.get_ns_and_schema(&d, &namespace, &db_schema);
                        namespace = Some(new_ns);
                        db_schema = new_schema;
                    } else {
                        let (sub_ns, sub_schema) = self.get_ns_and_schema(&d, &namespace, &db_schema);
                        if let Err(e) = self.lower_content(d.decls, Some(sub_ns), sub_schema) {
                            self.errors.push(e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn lower_struct(&mut self, s: ast::StructDecl, namespace: Option<String>, mut db_schema: Option<String>, full_name: &str) -> Result<HirStruct> {
        let id = *self.db.name_to_id.get(full_name).unwrap();
        let attrs = match self.lower_attrs(s.attrs) {
            Ok(a) => a,
            Err(e) => { self.errors.push(e); Vec::new() }
        };

        let mut layout = None;
        for attr in &attrs {
            if attr.name == "layout" {
                if !attr.args.is_empty() {
                    if let HirExprKind::Symbol(s) = &attr.args[0].value.kind {
                        layout = match s.as_str() {
                            "json" => Some(StructLayout::Json),
                            "table" => Some(StructLayout::Table),
                            _ => None,
                        };
                    }
                }
            }
        }

        // Check for @schema on struct
        for attr in &attrs {
            if attr.name == "schema" {
                if !attr.args.is_empty() {
                    if let HirExprKind::Literal(HirLiteral::String(s)) = &attr.args[0].value.kind {
                        db_schema = Some(s.clone());
                    }
                } else {
                    // @schema without args on struct? maybe use struct name? 
                    // Usually we use it on namespace. On struct it probably needs an arg.
                }
            }
        }

        let old_struct_id = self.current_struct_id;
        self.current_struct_id = Some(id);

        let mut fields = Vec::new();
        for f in s.fields {
            let f_attrs = match self.lower_attrs(f.attrs) {
                Ok(a) => a,
                Err(e) => { self.errors.push(e); Vec::new() }
            };
            let mut f_ty = match self.lower_type(f.ty, namespace.as_deref()) {
                Ok(t) => t,
                Err(e) => { self.errors.push(e); HirType::Unknown }
            };

            // Check for @relation
            let mut is_relation = false;
            let mut rel_name = None;
            let mut rel_fk = None;
            let mut rel_ref = None;
            for attr in &f_attrs {
                if attr.name == "relation" {
                    is_relation = true;
                    for arg in &attr.args {
                        match arg.name.as_deref() {
                            Some("name") => {
                                if let HirExprKind::Literal(HirLiteral::String(s)) = &arg.value.kind {
                                    rel_name = Some(s.clone());
                                }
                            }
                            Some("foreign_key") => {
                                if let HirExprKind::Literal(HirLiteral::String(s)) = &arg.value.kind {
                                    rel_fk = Some(s.clone());
                                }
                            }
                            Some("references") => {
                                if let HirExprKind::Literal(HirLiteral::String(s)) = &arg.value.kind {
                                    rel_ref = Some(s.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                    break;
                }
            }

            if is_relation {
                match &f_ty {
                    HirType::Struct(target_id) => {
                        f_ty = HirType::Relation { 
                            name: rel_name,
                            target: *target_id, 
                            is_list: false,
                            foreign_key: rel_fk,
                            references: rel_ref,
                        };
                    }
                    HirType::List(inner) => {
                        if let HirType::Struct(target_id) = inner.as_ref() {
                            f_ty = HirType::Relation { 
                                name: rel_name,
                                target: *target_id, 
                                is_list: true,
                                foreign_key: rel_fk,
                                references: rel_ref,
                            };
                        }
                    }
                    HirType::Optional(inner) => {
                        if let HirType::Struct(target_id) = inner.as_ref() {
                            f_ty = HirType::Relation { 
                                name: rel_name,
                                target: *target_id, 
                                is_list: false,
                                foreign_key: rel_fk,
                                references: rel_ref,
                            };
                        }
                    }
                    _ => {
                        // Not a struct or list of structs, but has @relation? 
                        // Error should probably be handled here or later.
                    }
                }
            }

            fields.push(HirField {
                attrs: f_attrs,
                name: f.name.name,
                ty: f_ty,
                span: f.span,
            });
        }
        self.current_struct_id = old_struct_id;
        Ok(HirStruct {
            id,
            attrs,
            name: s.name.name,
            namespace,
            schema: db_schema,
            layout,
            fields,
            span: s.span,
        })
    }

    fn lower_enum(&mut self, e: ast::EnumDecl, namespace: Option<String>, db_schema: Option<String>, full_name: &str) -> Result<HirEnum> {
        let id = *self.db.name_to_id.get(full_name).unwrap();
        let attrs = match self.lower_attrs(e.attrs) {
            Ok(a) => a,
            Err(e) => { self.errors.push(e); Vec::new() }
        };

        let mut layout = None;
        for attr in &attrs {
            if attr.name == "layout" {
                if !attr.args.is_empty() {
                    // Try to parse the first argument as a type
                    // In KQL, @layout(i32) means the argument is an expression that looks like a type
                    // But our parser might parse it as a symbol or a type.
                    // Let's check how @layout(i32) is parsed.
                    // If it's HirExprKind::Symbol("i32"), we can map it.
                    if let HirExprKind::Symbol(s) = &attr.args[0].value.kind {
                        layout = match s.as_str() {
                            "i8" => Some(HirType::Primitive(PrimitiveType::I8)),
                            "i16" => Some(HirType::Primitive(PrimitiveType::I16)),
                            "i32" => Some(HirType::Primitive(PrimitiveType::I32)),
                            "i64" => Some(HirType::Primitive(PrimitiveType::I64)),
                            "u8" => Some(HirType::Primitive(PrimitiveType::U8)),
                            "u16" => Some(HirType::Primitive(PrimitiveType::U16)),
                            "u32" => Some(HirType::Primitive(PrimitiveType::U32)),
                            "u64" => Some(HirType::Primitive(PrimitiveType::U64)),
                            "String" | "string" | "varchar" => Some(HirType::Primitive(PrimitiveType::String)),
                            _ => None,
                        };
                    }
                }
            }
        }

        let mut variants = Vec::new();
        for v in e.variants {
            let v_attrs = match self.lower_attrs(v.attrs) {
                Ok(a) => a,
                Err(e) => { self.errors.push(e); Vec::new() }
            };
            let fields = if let Some(f_vec) = v.fields {
                let mut hir_f_vec = Vec::new();
                for f in f_vec {
                    let f_attrs = match self.lower_attrs(f.attrs) {
                        Ok(a) => a,
                        Err(e) => { self.errors.push(e); Vec::new() }
                    };
                    let f_ty = match self.lower_type(f.ty, namespace.as_deref()) {
                        Ok(t) => t,
                        Err(e) => { self.errors.push(e); HirType::Unknown }
                    };
                    hir_f_vec.push(HirField {
                        attrs: f_attrs,
                        name: f.name.name,
                        ty: f_ty,
                        span: f.span,
                    });
                }
                Some(hir_f_vec)
            } else {
                None
            };
            variants.push(HirVariant {
                attrs: v_attrs,
                name: v.name.name,
                fields,
                span: v.span,
            });
        }
        Ok(HirEnum {
            id,
            attrs,
            name: e.name.name,
            namespace,
            schema: db_schema,
            layout,
            variants,
            span: e.span,
        })
    }

    fn lower_let(&mut self, l: ast::LetDecl, namespace: Option<String>, full_name: &str) -> Result<HirLet> {
        let id = *self.db.name_to_id.get(full_name).unwrap();
        let attrs = match self.lower_attrs(l.attrs) {
            Ok(a) => a,
            Err(e) => { self.errors.push(e); Vec::new() }
        };
        let value = match self.lower_expr(l.value) {
            Ok(v) => v,
            Err(e) => { 
                self.errors.push(e); 
                HirExpr { kind: HirExprKind::Literal(HirLiteral::Integer64(0)), ty: HirType::Unknown, span: l.span }
            }
        };
        let ty = if let Some(ast_ty) = l.ty {
            match self.lower_type(ast_ty, namespace.as_deref()) {
                Ok(t) => t,
                Err(e) => { self.errors.push(e); HirType::Unknown }
            }
        } else {
            value.ty.clone()
        };

        if ty != HirType::Unknown && value.ty != HirType::Unknown {
            if !self.can_assign(&ty, &value.ty) {
                self.errors.push(KqlError::semantic(
                    l.span,
                    format!(
                        "Type mismatch in let binding: expected {:?}, found {:?}",
                        ty, value.ty
                    ),
                ));
            }
        }

        Ok(HirLet {
            id,
            attrs,
            name: l.name.name,
            namespace,
            ty,
            value,
            span: l.span,
        })
    }

    fn lower_type_alias(&mut self, t: ast::TypeAliasDecl, namespace: Option<String>, full_name: &str) -> Result<HirTypeAlias> {
        let id = *self.db.name_to_id.get(full_name).unwrap();
        let attrs = match self.lower_attrs(t.attrs) {
            Ok(a) => a,
            Err(e) => { self.errors.push(e); Vec::new() }
        };
        let ty = self.lower_type(t.ty, namespace.as_deref())?;

        Ok(HirTypeAlias {
            id,
            attrs,
            name: t.name.name,
            namespace,
            ty,
            span: t.span,
        })
    }

    fn lower_attrs(&mut self, attrs: Vec<ast::Attribute>) -> Result<Vec<HirAttribute>> {
        let mut hir_attrs = Vec::new();
        for attr in attrs {
            let mut args = Vec::new();
            if let Some(ast_args) = attr.args {
                for arg in ast_args {
                    args.push(HirAttributeArg {
                        name: arg.name.map(|n| n.name),
                        value: self.lower_expr(arg.value)?,
                    });
                }
            }
            hir_attrs.push(HirAttribute {
                name: attr.name.name,
                args,
                span: attr.span,
            });
        }
        Ok(hir_attrs)
    }

    fn lower_type(&mut self, ty: ast::Type, namespace: Option<&str>) -> Result<HirType> {
        match ty {
            ast::Type::Named(n) => {
                // Special handling for Key<T>
                if n.name == "Key" {
                    if let Some(args) = n.args {
                        if args.len() == 1 {
                            let inner = self.lower_type(args[0].ty.clone(), namespace)?;
                            // Validate that inner is a primitive type or enum
                            match &inner {
                                HirType::Primitive(_) | HirType::Enum(_) => {}
                                _ => {
                                    self.errors.push(KqlError::semantic(
                                        n.span,
                                        "Key type argument must be a primitive type or an enum".to_string(),
                                    ));
                                }
                            }
                            return Ok(HirType::Key {
                                entity: None,
                                inner: Box::new(inner),
                            });
                        } else if args.len() == 2 {
                            let entity_ty = self.lower_type(args[0].ty.clone(), namespace)?;
                            let inner = self.lower_type(args[1].ty.clone(), namespace)?;
                            
                            let entity = if let HirType::Struct(id) = entity_ty {
                                Some(id)
                            } else {
                                self.errors.push(KqlError::semantic(
                                    n.span,
                                    "First argument to Key<Entity, T> must be a struct type".to_string(),
                                ));
                                None
                            };

                            match &inner {
                                HirType::Primitive(_) | HirType::Enum(_) => {}
                                _ => {
                                    self.errors.push(KqlError::semantic(
                                        n.span,
                                        "Second argument to Key<Entity, T> must be a primitive type or an enum".to_string(),
                                    ));
                                }
                            }

                            return Ok(HirType::Key {
                                entity,
                                inner: Box::new(inner),
                            });
                        }
                    }
                    return Err(KqlError::semantic(
                        n.span,
                        "Key type must have one or two generic arguments, e.g., Key<i32> or Key<User, i32>".to_string(),
                    ));
                }

                // Special handling for ForeignKey<name: T>
                if n.name == "ForeignKey" {
                    if let Some(args) = n.args {
                        if args.len() == 1 {
                            let arg = &args[0];
                            let entity_ty = self.lower_type(arg.ty.clone(), namespace)?;
                            if let HirType::Struct(id) = entity_ty {
                                return Ok(HirType::ForeignKey {
                                    name: arg.name.as_ref().map(|ident| ident.name.clone()),
                                    entity: id,
                                });
                            } else {
                                return Err(KqlError::semantic(
                                    n.span,
                                    "ForeignKey must reference a struct type".to_string(),
                                ));
                            }
                        }
                    }
                    return Err(KqlError::semantic(
                        n.span,
                        "ForeignKey type must have one generic argument, e.g., ForeignKey<User> or ForeignKey<author: User>".to_string(),
                    ));
                }

                // Special handling for List<T> as an alternative to [T]
                if n.name == "List" {
                    if let Some(args) = n.args {
                        if args.len() == 1 {
                            let inner = self.lower_type(args[0].ty.clone(), namespace)?;
                            return Ok(HirType::List(Box::new(inner)));
                        }
                    }
                }

                // Resolve type name
                if let Some(&id) = self.db.name_to_id.get(&n.name) {
                    return self.resolve_id_as_type(id, n.span);
                }

                // Try to resolve in current namespace
                if let Some(ns) = namespace {
                    let qualified_name = format!("{}::{}", ns, n.name);
                    if let Some(&id) = self.db.name_to_id.get(&qualified_name) {
                        return self.resolve_id_as_type(id, n.span);
                    }
                }

                // Handle primitive types
                match n.name.as_str() {
                    "i8" => Ok(HirType::Primitive(PrimitiveType::I8)),
                    "i16" => Ok(HirType::Primitive(PrimitiveType::I16)),
                    "i32" => Ok(HirType::Primitive(PrimitiveType::I32)),
                    "i64" => Ok(HirType::Primitive(PrimitiveType::I64)),
                    "u8" => Ok(HirType::Primitive(PrimitiveType::U8)),
                    "u16" => Ok(HirType::Primitive(PrimitiveType::U16)),
                    "u32" => Ok(HirType::Primitive(PrimitiveType::U32)),
                    "u64" => Ok(HirType::Primitive(PrimitiveType::U64)),
                    "f32" => Ok(HirType::Primitive(PrimitiveType::F32)),
                    "f64" => Ok(HirType::Primitive(PrimitiveType::F64)),
                    "String" | "string" => Ok(HirType::Primitive(PrimitiveType::String)),
                    "Bool" | "bool" | "boolean" => Ok(HirType::Primitive(PrimitiveType::Bool)),
                    "DateTime" => Ok(HirType::Primitive(PrimitiveType::DateTime)),
                    "Uuid" | "UUID" => Ok(HirType::Primitive(PrimitiveType::Uuid)),
                    "d64" | "D64" => Ok(HirType::Primitive(PrimitiveType::D64)),
                    "d128" | "D128" => Ok(HirType::Primitive(PrimitiveType::D128)),
                    _ => Err(KqlError::semantic(n.span, format!("Unknown type: {}", n.name))),
                }
            }
            ast::Type::List(l) => {
                let inner = self.lower_type(*l.inner, namespace)?;
                Ok(HirType::List(Box::new(inner)))
            }
            ast::Type::Optional(o) => {
                let inner = self.lower_type(*o.inner, namespace)?;
                Ok(HirType::Optional(Box::new(inner)))
            }
        }
    }

    fn resolve_id_as_type(&mut self, id: HirId, span: Span) -> Result<HirType> {
        if self.resolving_aliases.contains(&id) {
            return Err(KqlError::semantic(span, "Recursive type alias detected".to_string()));
        }

        let kind = self.db.id_to_kind.get(&id).cloned();
        match kind {
            Some(HirKind::Struct) => Ok(HirType::Struct(id)),
            Some(HirKind::Enum) => Ok(HirType::Enum(id)),
            Some(HirKind::TypeAlias) => {
                if let Some(alias) = self.db.type_aliases.get(&id) {
                    return Ok(alias.ty.clone());
                }

                // Lazy lowering of type alias
                if let Some(decl) = self.ast_decls.get(&id).cloned() {
                    if let ast::Decl::TypeAlias(t) = decl {
                        self.resolving_aliases.push(id);
                        
                        // We need the full name and namespace to lower it properly
                        // Let's find them from db.name_to_id
                        let mut full_name = String::new();
                        for (name, &name_id) in &self.db.name_to_id {
                            if name_id == id {
                                full_name = name.clone();
                                break;
                            }
                        }
                        
                        let namespace = if full_name.contains("::") {
                            let parts: Vec<&str> = full_name.split("::").collect();
                            Some(parts[..parts.len()-1].join("::"))
                        } else {
                            None
                        };

                        let result = self.lower_type_alias(t, namespace, &full_name);
                        self.resolving_aliases.pop();
                        
                        let hir_alias = result?;
                        self.db.type_aliases.insert(id, hir_alias.clone());
                        return Ok(hir_alias.ty);
                    }
                }
                Ok(HirType::Unknown)
            }
            _ => Ok(HirType::Unknown),
        }
    }

    fn lower_expr(&mut self, expr: ast::Expr) -> Result<HirExpr> {
        match expr {
            ast::Expr::Literal(l) => {
                let (kind, ty) = match l.kind {
                    ast::LiteralKind::Number(n) => {
                        if n.contains('.') {
                            (
                                HirExprKind::Literal(HirLiteral::Float64(n.parse().unwrap_or(0.0))),
                                HirType::Primitive(PrimitiveType::F32),
                            )
                        }
                        else {
                            (
                                HirExprKind::Literal(HirLiteral::Integer64(n.parse().unwrap_or(0))),
                                HirType::Primitive(PrimitiveType::I32),
                            )
                        }
                    }
                    ast::LiteralKind::String(s) => {
                        (HirExprKind::Literal(HirLiteral::String(s)), HirType::Primitive(PrimitiveType::String))
                    }
                    ast::LiteralKind::Boolean(b) => {
                        (HirExprKind::Literal(HirLiteral::Bool(b)), HirType::Primitive(PrimitiveType::Bool))
                    }
                    ast::LiteralKind::Null => {
                        (HirExprKind::Literal(HirLiteral::Null), HirType::Null)
                    }
                };
                Ok(HirExpr { kind, ty, span: l.span })
            }
            ast::Expr::Variable(v) => {
                // 1. Check current struct fields if in struct context
                if let Some(struct_id) = self.current_struct_id {
                    if let Some(s) = self.db.structs.get(&struct_id) {
                        if let Some(f) = s.fields.iter().find(|f| f.name == v.name) {
                            return Ok(HirExpr {
                                kind: HirExprKind::Member {
                                    object: Box::new(HirExpr {
                                        kind: HirExprKind::Symbol("this".to_string()),
                                        ty: HirType::Struct(struct_id),
                                        span: v.span,
                                    }),
                                    member: v.name.clone(),
                                },
                                ty: f.ty.clone(),
                                span: v.span,
                            });
                        }
                    }
                }

                // 2. Check global scope
                if let Some(&id) = self.db.name_to_id.get(&v.name) {
                    let ty = if let Some(l) = self.db.lets.get(&id) {
                        l.ty.clone()
                    }
                    else if let Some(kind) = self.db.id_to_kind.get(&id) {
                        match kind {
                            HirKind::Struct => HirType::Struct(id),
                            HirKind::Enum => HirType::Enum(id),
                            HirKind::TypeAlias => {
                                if let Some(alias) = self.db.type_aliases.get(&id) {
                                    alias.ty.clone()
                                } else {
                                    HirType::Unknown
                                }
                            }
                            HirKind::Let => HirType::Unknown,
                        }
                    }
                    else {
                        HirType::Unknown
                    };
                    Ok(HirExpr { kind: HirExprKind::Variable(id), ty, span: v.span })
                }
                else {
                    // If not found in global scope, treat as a symbol (might be a field name)
                    Ok(HirExpr { kind: HirExprKind::Symbol(v.name), ty: HirType::Unknown, span: v.span })
                }
            }
            ast::Expr::Binary(b) => {
                let mut left = self.lower_expr(*b.left)?;
                let mut right = self.lower_expr(*b.right)?;
                let op = match b.op.kind {
                    ast::BinaryOpKind::Add => HirBinaryOp::Add,
                    ast::BinaryOpKind::Sub => HirBinaryOp::Sub,
                    ast::BinaryOpKind::Mul => HirBinaryOp::Mul,
                    ast::BinaryOpKind::Div => HirBinaryOp::Div,
                    ast::BinaryOpKind::Mod => HirBinaryOp::Mod,
                    ast::BinaryOpKind::Eq => HirBinaryOp::Eq,
                    ast::BinaryOpKind::NotEq => HirBinaryOp::NotEq,
                    ast::BinaryOpKind::Gt => HirBinaryOp::Gt,
                    ast::BinaryOpKind::Lt => HirBinaryOp::Lt,
                    ast::BinaryOpKind::GtEq => HirBinaryOp::GtEq,
                    ast::BinaryOpKind::LtEq => HirBinaryOp::LtEq,
                    ast::BinaryOpKind::And => HirBinaryOp::And,
                    ast::BinaryOpKind::Or => HirBinaryOp::Or,
                };

                let ty = match self.check_binary_op(&left, op, &right, b.span) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        HirType::Unknown
                    }
                };

                // Insert implicit casts if needed
                if ty != HirType::Unknown {
                    match op {
                        HirBinaryOp::Add | HirBinaryOp::Sub | HirBinaryOp::Mul | HirBinaryOp::Div | HirBinaryOp::Mod => {
                            if left.ty != ty {
                                let span = left.span;
                                left = HirExpr {
                                    kind: HirExprKind::Cast { expr: Box::new(left), target_ty: ty.clone() },
                                    ty: ty.clone(),
                                    span,
                                };
                            }
                            if right.ty != ty {
                                let span = right.span;
                                right = HirExpr {
                                    kind: HirExprKind::Cast { expr: Box::new(right), target_ty: ty.clone() },
                                    ty: ty.clone(),
                                    span,
                                };
                            }
                        }
                        HirBinaryOp::Eq | HirBinaryOp::NotEq | HirBinaryOp::Gt | HirBinaryOp::Lt | HirBinaryOp::GtEq | HirBinaryOp::LtEq => {
                            if let Some(common_ty) = self.promote_numeric_types(&left.ty, &right.ty) {
                                if left.ty != common_ty {
                                    let span = left.span;
                                    left = HirExpr {
                                        kind: HirExprKind::Cast { expr: Box::new(left), target_ty: common_ty.clone() },
                                        ty: common_ty.clone(),
                                        span,
                                    };
                                }
                                if right.ty != common_ty {
                                    let span = right.span;
                                    right = HirExpr {
                                        kind: HirExprKind::Cast { expr: Box::new(right), target_ty: common_ty.clone() },
                                        ty: common_ty.clone(),
                                        span,
                                    };
                                }
                            }
                        }
                        _ => {}
                    }
                }

                Ok(HirExpr { kind: HirExprKind::Binary { left: Box::new(left), op, right: Box::new(right) }, ty, span: b.span })
            }
            ast::Expr::Unary(u) => {
                let expr = self.lower_expr(*u.expr)?;
                let op = match u.op.kind {
                    ast::UnaryOpKind::Neg => HirUnaryOp::Neg,
                    ast::UnaryOpKind::Not => HirUnaryOp::Not,
                };

                let ty = match self.check_unary_op(op, &expr, u.span) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        HirType::Unknown
                    }
                };
                Ok(HirExpr { kind: HirExprKind::Unary { op, expr: Box::new(expr) }, ty, span: u.span })
            }
            ast::Expr::Call(c) => {
                let func = self.lower_expr(*c.func)?;
                let mut args = Vec::new();
                for a in c.args {
                    args.push(self.lower_expr(a)?);
                }
                
                let ty = match &func.ty {
                    HirType::Struct(id) => {
                        // Validate arguments against struct fields
                        if let Some(s) = self.db.structs.get(id) {
                            if args.len() != s.fields.len() {
                                self.errors.push(KqlError::semantic(
                                    c.span,
                                    format!("Struct '{}' expects {} arguments, but {} were provided", s.name, s.fields.len(), args.len()),
                                ));
                            } else {
                                for (i, (arg, field)) in args.iter().zip(s.fields.iter()).enumerate() {
                                    if !self.can_assign(&field.ty, &arg.ty) {
                                        self.errors.push(KqlError::semantic(
                                            arg.span,
                                            format!("Argument {} to struct '{}' has type {:?}, but field '{}' expects {:?}", i + 1, s.name, arg.ty, field.name, field.ty),
                                        ));
                                    }
                                }
                            }
                        }
                        HirType::Struct(*id)
                    }
                    HirType::Enum(id) => {
                        // Validate arguments against enum variant fields
                        if let HirExprKind::Member { member, .. } = &func.kind {
                            if let Some(e) = self.db.enums.get(id) {
                                if let Some(v) = e.variants.iter().find(|v| v.name == *member) {
                                    if let Some(fields) = &v.fields {
                                        if args.len() != fields.len() {
                                            self.errors.push(KqlError::semantic(
                                                c.span,
                                                format!("Enum variant '{}::{}' expects {} arguments, but {} were provided", e.name, v.name, fields.len(), args.len()),
                                            ));
                                        } else {
                                            for (i, (arg, field)) in args.iter().zip(fields.iter()).enumerate() {
                                                if !self.can_assign(&field.ty, &arg.ty) {
                                                    self.errors.push(KqlError::semantic(
                                                        arg.span,
                                                        format!("Argument {} to variant '{}::{}' has type {:?}, but field '{}' expects {:?}", i + 1, e.name, v.name, arg.ty, field.name, field.ty),
                                                    ));
                                                }
                                            }
                                        }
                                    } else if !args.is_empty() {
                                        self.errors.push(KqlError::semantic(
                                            c.span,
                                            format!("Enum variant '{}::{}' expects 0 arguments, but {} were provided", e.name, v.name, args.len()),
                                        ));
                                    }
                                }
                            }
                        }
                        HirType::Enum(*id)
                    }
                    _ => {
                        // Check for built-in functions
                        if let HirExprKind::Symbol(name) = &func.kind {
                            if let Some(ret_ty) = self.check_builtin_function(name, &args) {
                                ret_ty
                            } else {
                                HirType::Unknown
                            }
                        } else {
                            HirType::Unknown
                        }
                    }
                };
                
                Ok(HirExpr { kind: HirExprKind::Call { func: Box::new(func), args }, ty, span: c.span })
            }
            ast::Expr::Member(m) => {
                let object = self.lower_expr(*m.object)?;
                let member_name = m.member.name.clone();
                let mut ty = HirType::Unknown;

                match &object.ty {
                    HirType::Struct(id) => {
                        if let Some(s) = self.db.structs.get(id) {
                            if let Some(f) = s.fields.iter().find(|f| f.name == member_name) {
                                ty = f.ty.clone();
                            } else {
                                self.errors.push(KqlError::semantic(
                                    m.member.span,
                                    format!("Struct '{}' has no field '{}'", s.name, member_name),
                                ));
                            }
                        }
                    }
                    HirType::Enum(id) => {
                        // For enum variants, the type is the enum itself if it's a variant
                        if let Some(e) = self.db.enums.get(id) {
                            if let Some(_v) = e.variants.iter().find(|v| v.name == member_name) {
                                ty = HirType::Enum(*id);
                            } else {
                                self.errors.push(KqlError::semantic(
                                    m.member.span,
                                    format!("Enum '{}' has no variant '{}'", e.name, member_name),
                                ));
                            }
                        }
                    }
                    HirType::Optional(inner) => {
                        if let HirType::Struct(id) = inner.as_ref() {
                            if let Some(s) = self.db.structs.get(id) {
                                if let Some(f) = s.fields.iter().find(|f| f.name == member_name) {
                                    ty = HirType::Optional(Box::new(f.ty.clone()));
                                } else {
                                    self.errors.push(KqlError::semantic(
                                        m.member.span,
                                        format!("Struct '{}' has no field '{}'", s.name, member_name),
                                    ));
                                }
                            }
                        }
                    }
                    HirType::Relation { target, is_list, .. } => {
                        if member_name == "count" {
                             ty = HirType::Primitive(PrimitiveType::I64);
                        } else if let Some(s) = self.db.structs.get(target) {
                            if let Some(f) = s.fields.iter().find(|f| f.name == member_name) {
                                ty = f.ty.clone();
                                if *is_list {
                                    ty = HirType::List(Box::new(ty));
                                }
                            } else {
                                self.errors.push(KqlError::semantic(
                                    m.member.span,
                                    format!("Struct '{}' has no field '{}'", s.name, member_name),
                                ));
                            }
                        }
                    }
                    HirType::List(_) => {
                        if member_name == "count" {
                            ty = HirType::Primitive(PrimitiveType::I64);
                        } else {
                            self.errors.push(KqlError::semantic(
                                m.span,
                                format!("Cannot access member '{}' on List type", member_name),
                            ));
                        }
                    }
                    HirType::Unknown => {}
                    _ => {
                        self.errors.push(KqlError::semantic(
                            m.span,
                            format!("Cannot access member '{}' on type {:?}", member_name, object.ty),
                        ));
                    }
                }

                Ok(HirExpr {
                    kind: HirExprKind::Member {
                        object: Box::new(object),
                        member: member_name,
                    },
                    ty,
                    span: m.span,
                })
            }
        }
    }

    fn check_builtin_function(&self, name: &str, args: &[HirExpr]) -> Option<HirType> {
        match name {
            "now" | "current_timestamp" => {
                if args.is_empty() {
                    Some(HirType::Primitive(PrimitiveType::DateTime))
                } else {
                    None
                }
            }
            "uuid" | "gen_random_uuid" => {
                if args.is_empty() {
                    Some(HirType::Primitive(PrimitiveType::Uuid))
                } else {
                    None
                }
            }
            "count" => Some(HirType::Primitive(PrimitiveType::I64)),
            "sum" | "min" | "max" => {
                if args.len() == 1 {
                    let arg_ty = &args[0].ty;
                    if matches!(arg_ty, HirType::Primitive(PrimitiveType::I32 | PrimitiveType::I64 | PrimitiveType::F32 | PrimitiveType::F64 | PrimitiveType::D128)) {
                        Some(arg_ty.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "avg" => {
                if args.len() == 1 {
                    let arg_ty = &args[0].ty;
                    if matches!(arg_ty, HirType::Primitive(PrimitiveType::I32 | PrimitiveType::I64 | PrimitiveType::F32 | PrimitiveType::F64 | PrimitiveType::D128)) {
                        Some(HirType::Primitive(PrimitiveType::F64)) // Always return float for average
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "upper" | "lower" | "trim" | "concat" => {
                Some(HirType::Primitive(PrimitiveType::String))
            }
            "abs" => {
                if args.len() == 1 {
                    let arg_ty = &args[0].ty;
                    if matches!(arg_ty, HirType::Primitive(PrimitiveType::I32 | PrimitiveType::I64 | PrimitiveType::F32 | PrimitiveType::F64 | PrimitiveType::D128)) {
                        Some(arg_ty.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "coalesce" | "ifnull" => {
                if args.len() >= 2 {
                    let common_ty = args[0].ty.clone();
                    // Strip optional if needed for comparison
                    let mut base_ty = match &common_ty {
                        HirType::Optional(inner) => inner.as_ref().clone(),
                        _ => common_ty.clone(),
                    };

                    for arg in &args[1..] {
                        let arg_base_ty = match &arg.ty {
                            HirType::Optional(inner) => inner.as_ref().clone(),
                            _ => arg.ty.clone(),
                        };
                        
                        if let Some(promoted) = self.promote_numeric_types(&base_ty, &arg_base_ty) {
                            base_ty = promoted;
                        } else if base_ty != arg_base_ty && arg.ty != HirType::Null {
                            return None; // Incompatible types
                        }
                    }
                    Some(base_ty)
                } else if args.len() == 1 {
                    Some(args[0].ty.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn check_binary_op(&self, left: &HirExpr, op: HirBinaryOp, right: &HirExpr, span: Span) -> Result<HirType> {
        if left.ty == HirType::Unknown || right.ty == HirType::Unknown {
            return Ok(HirType::Unknown);
        }

        match op {
            HirBinaryOp::Add | HirBinaryOp::Sub | HirBinaryOp::Mul | HirBinaryOp::Div | HirBinaryOp::Mod => {
                if let Some(ty) = self.promote_numeric_types(&left.ty, &right.ty) {
                    Ok(ty)
                }
                else {
                    Err(KqlError::semantic(
                        span,
                        format!("Cannot apply arithmetic operator {:?} to {:?} and {:?}", op, left.ty, right.ty),
                    ))
                }
            }
            HirBinaryOp::Eq | HirBinaryOp::NotEq => {
                if left.ty == right.ty 
                    || self.promote_numeric_types(&left.ty, &right.ty).is_some()
                    || (left.ty == HirType::Null && matches!(right.ty, HirType::Optional(_)))
                    || (right.ty == HirType::Null && matches!(left.ty, HirType::Optional(_)))
                {
                    Ok(HirType::Primitive(PrimitiveType::Bool))
                }
                else {
                    Err(KqlError::semantic(span, format!("Cannot compare {:?} and {:?}", left.ty, right.ty)))
                }
            }
            HirBinaryOp::Gt | HirBinaryOp::Lt | HirBinaryOp::GtEq | HirBinaryOp::LtEq => {
                if left.ty == right.ty || self.promote_numeric_types(&left.ty, &right.ty).is_some() {
                    Ok(HirType::Primitive(PrimitiveType::Bool))
                }
                else {
                    Err(KqlError::semantic(span, format!("Cannot compare {:?} and {:?}", left.ty, right.ty)))
                }
            }
            HirBinaryOp::And | HirBinaryOp::Or => {
                if left.ty == HirType::Primitive(PrimitiveType::Bool) && right.ty == HirType::Primitive(PrimitiveType::Bool) {
                    Ok(HirType::Primitive(PrimitiveType::Bool))
                }
                else {
                    Err(KqlError::semantic(
                        span,
                        format!("Logical operators require boolean operands, found {:?} and {:?}", left.ty, right.ty),
                    ))
                }
            }
        }
    }

    fn check_unary_op(&self, op: HirUnaryOp, expr: &HirExpr, span: Span) -> Result<HirType> {
        if expr.ty == HirType::Unknown {
            return Ok(HirType::Unknown);
        }

        match op {
            HirUnaryOp::Neg => {
                if matches!(expr.ty, HirType::Primitive(PrimitiveType::I32 | PrimitiveType::I64 | PrimitiveType::F32 | PrimitiveType::F64 | PrimitiveType::D128)) {
                    Ok(expr.ty.clone())
                }
                else {
                    Err(KqlError::semantic(span, format!("Unary negation requires numeric type, found {:?}", expr.ty)))
                }
            }
            HirUnaryOp::Not => {
                if expr.ty == HirType::Primitive(PrimitiveType::Bool) {
                    Ok(HirType::Primitive(PrimitiveType::Bool))
                }
                else {
                    Err(KqlError::semantic(span, format!("Unary NOT requires boolean type, found {:?}", expr.ty)))
                }
            }
        }
    }

    fn can_assign(&self, target: &HirType, source: &HirType) -> bool {
        if target == source {
            return true;
        }

        if source == &HirType::Null && matches!(target, HirType::Optional(_)) {
            return true;
        }

        match (target, source) {
            (HirType::Primitive(p_t), HirType::Primitive(p_s)) => {
                match (p_t, p_s) {
                    // Integer promotion
                    (PrimitiveType::I64, PrimitiveType::I32) => true,
                    // Float promotion
                    (PrimitiveType::F64, PrimitiveType::F32) => true,
                    // Integer to Float
                    (PrimitiveType::F32, PrimitiveType::I32) => true,
                    (PrimitiveType::F64, PrimitiveType::I32) => true,
                    (PrimitiveType::F64, PrimitiveType::I64) => true,
                    // For now, allow I64 to I32 because all integer literals are I64
                    (PrimitiveType::I32, PrimitiveType::I64) => true,
                    _ => false,
                }
            }
            // Key<T> can be assigned from T
            (HirType::Key { inner: t_inner, .. }, source_ty) => {
                self.can_assign(t_inner, source_ty)
            }
            // ForeignKey<T> can be assigned from the Key type of T
            (HirType::ForeignKey { entity: target_id, .. }, HirType::Key { entity: Some(source_id), .. }) => {
                target_id == source_id
            }
            // ForeignKey<T> can also be assigned from the inner type of the Key of T (e.g. i32)
            (HirType::ForeignKey { entity: target_id, .. }, source_ty) => {
                if let Some(s) = self.db.structs.get(target_id) {
                    if let Some(pk_field) = s.fields.iter().find(|f| matches!(f.ty, HirType::Key { .. })) {
                        if let HirType::Key { inner, .. } = &pk_field.ty {
                            return self.can_assign(inner, source_ty);
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn promote_numeric_types(&self, t1: &HirType, t2: &HirType) -> Option<HirType> {
        match (t1, t2) {
            (HirType::Primitive(p1), HirType::Primitive(p2)) => {
                match (p1, p2) {
                    (PrimitiveType::I32, PrimitiveType::I32) => Some(HirType::Primitive(PrimitiveType::I32)),
                    (PrimitiveType::I64, PrimitiveType::I64) => Some(HirType::Primitive(PrimitiveType::I64)),
                    (PrimitiveType::F32, PrimitiveType::F32) => Some(HirType::Primitive(PrimitiveType::F32)),
                    (PrimitiveType::F64, PrimitiveType::F64) => Some(HirType::Primitive(PrimitiveType::F64)),
                    (PrimitiveType::D128, PrimitiveType::D128) => Some(HirType::Primitive(PrimitiveType::D128)),

                    // Integer promotion
                    (PrimitiveType::I32, PrimitiveType::I64) | (PrimitiveType::I64, PrimitiveType::I32) => {
                        Some(HirType::Primitive(PrimitiveType::I64))
                    }

                    // Float promotion
                    (PrimitiveType::F32, PrimitiveType::F64) | (PrimitiveType::F64, PrimitiveType::F32) => {
                        Some(HirType::Primitive(PrimitiveType::F64))
                    }

                    // Integer to Float
                    (PrimitiveType::I32, PrimitiveType::F32) | (PrimitiveType::F32, PrimitiveType::I32) => {
                        Some(HirType::Primitive(PrimitiveType::F32))
                    }
                    (PrimitiveType::I32, PrimitiveType::F64) | (PrimitiveType::F64, PrimitiveType::I32) => {
                        Some(HirType::Primitive(PrimitiveType::F64))
                    }
                    (PrimitiveType::I64, PrimitiveType::F32) | (PrimitiveType::F32, PrimitiveType::I64) => {
                        Some(HirType::Primitive(PrimitiveType::F32))
                    }
                    (PrimitiveType::I64, PrimitiveType::F64) | (PrimitiveType::F64, PrimitiveType::I64) => {
                        Some(HirType::Primitive(PrimitiveType::F64))
                    }

                    // D128 promotion (always stays D128 if one is D128)
                    (PrimitiveType::D128, _) | (_, PrimitiveType::D128) => {
                        if matches!(p1, PrimitiveType::I32 | PrimitiveType::I64 | PrimitiveType::F32 | PrimitiveType::F64 | PrimitiveType::D128) &&
                           matches!(p2, PrimitiveType::I32 | PrimitiveType::I64 | PrimitiveType::F32 | PrimitiveType::F64 | PrimitiveType::D128) {
                            Some(HirType::Primitive(PrimitiveType::D128))
                        } else {
                            None
                        }
                    }

                    _ => None,
                }
            }
            _ => None,
        }
    }
}
