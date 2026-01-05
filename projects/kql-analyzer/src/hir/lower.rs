use super::*;
use kql_ast as ast;
use kql_types::{KqlError, Result, Span};

pub struct Lowerer {
    pub db: HirProgram,
}

impl Lowerer {
    pub fn new() -> Self {
        Self { db: HirProgram::default() }
    }

    pub fn lower_program(&mut self, ast_db: &ast::Database) -> Result<HirProgram> {
        self.lower_decls(ast_db.decls.clone())?;
        Ok(self.db.clone())
    }

    pub fn lower_decls(&mut self, decls: Vec<ast::Decl>) -> Result<()> {
        self.collect_names(decls.clone(), None, None)?;
        self.lower_content(decls, None, None)
    }

    fn collect_names(&mut self, decls: Vec<ast::Decl>, namespace: Option<String>, db_schema: Option<String>) -> Result<()> {
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
                }
                ast::Decl::Namespace(d) => {
                    let sub_namespace = if let Some(ns) = &namespace {
                        format!("{}::{}", ns, d.name.name)
                    } else {
                        d.name.name.clone()
                    };

                    if namespace.is_some() && !d.is_block {
                         return Err(KqlError::parse(d.span, "Top-level namespace cannot be nested within another namespace. Use block-style 'namespace { ... }' instead.".to_string()));
                    }
                    
                    // Extract @schema from attributes
                    let mut sub_db_schema = db_schema.clone();
                    for attr in &d.attrs {
                        if attr.name.name == "schema" {
                            if let Some(args) = &attr.args {
                                if !args.is_empty() {
                                    if let ast::Expr::Literal(ast::LiteralExpr { kind: ast::LiteralKind::String(s), .. }) = &args[0].value {
                                        sub_db_schema = Some(s.clone());
                                    }
                                }
                            }
                        }
                    }
                    
                    self.collect_names(d.decls.clone(), Some(sub_namespace), sub_db_schema)?;
                }
            }
        }
        Ok(())
    }

    fn lower_content(&mut self, decls: Vec<ast::Decl>, namespace: Option<String>, db_schema: Option<String>) -> Result<()> {
        for decl in decls {
            match decl {
                ast::Decl::Struct(s) => {
                    let full_name = if let Some(ns) = &namespace {
                        format!("{}::{}", ns, s.name.name)
                    } else {
                        s.name.name.clone()
                    };
                    let hir_s = self.lower_struct(s, namespace.clone(), db_schema.clone(), &full_name)?;
                    self.db.structs.insert(hir_s.id, hir_s);
                }
                ast::Decl::Enum(e) => {
                    let full_name = if let Some(ns) = &namespace {
                        format!("{}::{}", ns, e.name.name)
                    } else {
                        e.name.name.clone()
                    };
                    let hir_e = self.lower_enum(e, namespace.clone(), db_schema.clone(), &full_name)?;
                    self.db.enums.insert(hir_e.id, hir_e);
                }
                ast::Decl::Let(l) => {
                    let full_name = if let Some(ns) = &namespace {
                        format!("{}::{}", ns, l.name.name)
                    } else {
                        l.name.name.clone()
                    };
                    let hir_l = self.lower_let(l, namespace.clone(), &full_name)?;
                    self.db.lets.insert(hir_l.id, hir_l);
                }
                ast::Decl::Namespace(d) => {
                    let sub_namespace = if let Some(ns) = &namespace {
                        format!("{}::{}", ns, d.name.name)
                    } else {
                        d.name.name.clone()
                    };

                    // Extract @schema from attributes
                    let mut sub_db_schema = db_schema.clone();
                    for attr in &d.attrs {
                        if attr.name.name == "schema" {
                            if let Some(args) = &attr.args {
                                if !args.is_empty() {
                                    if let ast::Expr::Literal(ast::LiteralExpr { kind: ast::LiteralKind::String(s), .. }) = &args[0].value {
                                        sub_db_schema = Some(s.clone());
                                    }
                                }
                            }
                        }
                    }
                    self.lower_content(d.decls.clone(), Some(sub_namespace), sub_db_schema)?;
                }
            }
        }
        Ok(())
    }

    fn lower_struct(&mut self, s: ast::StructDecl, namespace: Option<String>, db_schema: Option<String>, full_name: &str) -> Result<HirStruct> {
        let id = *self.db.name_to_id.get(full_name).unwrap();
        let attrs = self.lower_attrs(s.attrs)?;
        let mut fields = Vec::new();
        for f in s.fields {
            fields.push(HirField {
                attrs: self.lower_attrs(f.attrs)?,
                name: f.name.name,
                ty: self.lower_type(f.ty, namespace.as_deref())?,
                span: f.span,
            });
        }
        Ok(HirStruct {
            id,
            attrs,
            name: s.name.name,
            namespace,
            schema: db_schema,
            fields,
            span: s.span,
        })
    }

    fn lower_enum(&mut self, e: ast::EnumDecl, namespace: Option<String>, db_schema: Option<String>, full_name: &str) -> Result<HirEnum> {
        let id = *self.db.name_to_id.get(full_name).unwrap();
        let attrs = self.lower_attrs(e.attrs)?;
        let mut variants = Vec::new();
        for v in e.variants {
            let fields = if let Some(f_vec) = v.fields {
                let mut hir_f_vec = Vec::new();
                for f in f_vec {
                    hir_f_vec.push(HirField {
                        attrs: self.lower_attrs(f.attrs)?,
                        name: f.name.name,
                        ty: self.lower_type(f.ty, namespace.as_deref())?,
                        span: f.span,
                    });
                }
                Some(hir_f_vec)
            } else {
                None
            };
            variants.push(HirVariant {
                attrs: self.lower_attrs(v.attrs)?,
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
            variants,
            span: e.span,
        })
    }

    fn lower_let(&mut self, l: ast::LetDecl, namespace: Option<String>, full_name: &str) -> Result<HirLet> {
        let id = *self.db.name_to_id.get(full_name).unwrap();
        let attrs = self.lower_attrs(l.attrs)?;
        let value = self.lower_expr(l.value)?;
        let ty = if let Some(ast_ty) = l.ty {
            self.lower_type(ast_ty, namespace.as_deref())?
        } else {
            value.ty.clone()
        };

        // Type checking for let assignment
        if ty != HirType::Unknown && value.ty != HirType::Unknown && ty != value.ty {
            return Err(KqlError::semantic(
                l.span,
                format!(
                    "Type mismatch in let binding: expected {:?}, found {:?}",
                    ty, value.ty
                ),
            ));
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
                            let inner = self.lower_type(args[0].clone(), namespace)?;
                            return Ok(HirType::Key {
                                entity: None,
                                inner: Box::new(inner),
                            });
                        } else if args.len() == 2 {
                            let entity_ty = self.lower_type(args[0].clone(), namespace)?;
                            let inner = self.lower_type(args[1].clone(), namespace)?;
                            let entity = if let HirType::Struct(id) = entity_ty {
                                Some(id)
                            } else {
                                None
                            };
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

                // Special handling for List<T> as an alternative to [T]
                if n.name == "List" {
                    if let Some(args) = n.args {
                        if args.len() == 1 {
                            let inner = self.lower_type(args[0].clone(), namespace)?;
                            return Ok(HirType::List(Box::new(inner)));
                        }
                    }
                }

                // Resolve type name
                if let Some(id) = self.db.name_to_id.get(&n.name) {
                    let kind = self.db.id_to_kind.get(id).unwrap();
                    return match kind {
                        HirKind::Struct => Ok(HirType::Struct(*id)),
                        HirKind::Enum => Ok(HirType::Enum(*id)),
                        HirKind::Let => Err(KqlError::semantic(n.span, format!("{} is a variable, not a type", n.name))),
                    };
                }

                // Try to resolve in current namespace
                if let Some(ns) = namespace {
                    let qualified_name = format!("{}::{}", ns, n.name);
                    if let Some(id) = self.db.name_to_id.get(&qualified_name) {
                        let kind = self.db.id_to_kind.get(id).unwrap();
                        return match kind {
                            HirKind::Struct => Ok(HirType::Struct(*id)),
                            HirKind::Enum => Ok(HirType::Enum(*id)),
                            HirKind::Let => Err(KqlError::semantic(n.span, format!("{} is a variable, not a type", qualified_name))),
                        };
                    }
                }

                // Handle primitive types
                match n.name.as_str() {
                    "i32" => Ok(HirType::Primitive(PrimitiveType::I32)),
                    "i64" => Ok(HirType::Primitive(PrimitiveType::I64)),
                    "f32" => Ok(HirType::Primitive(PrimitiveType::F32)),
                    "f64" => Ok(HirType::Primitive(PrimitiveType::F64)),
                    "String" => Ok(HirType::Primitive(PrimitiveType::String)),
                    "Bool" | "bool" => Ok(HirType::Primitive(PrimitiveType::Bool)),
                    "DateTime" => Ok(HirType::Primitive(PrimitiveType::DateTime)),
                    "Uuid" | "UUID" => Ok(HirType::Primitive(PrimitiveType::Uuid)),
                    "D128" | "d128" => Ok(HirType::Primitive(PrimitiveType::D128)),
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
                };
                Ok(HirExpr { kind, ty, span: l.span })
            }
            ast::Expr::Variable(v) => {
                if let Some(&id) = self.db.name_to_id.get(&v.name) {
                    let ty = if let Some(l) = self.db.lets.get(&id) {
                        l.ty.clone()
                    }
                    else if let Some(kind) = self.db.id_to_kind.get(&id) {
                        match kind {
                            HirKind::Struct => HirType::Struct(id),
                            HirKind::Enum => HirType::Enum(id),
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
                let left = self.lower_expr(*b.left)?;
                let right = self.lower_expr(*b.right)?;
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

                let ty = self.check_binary_op(&left, op, &right, b.span)?;
                Ok(HirExpr { kind: HirExprKind::Binary { left: Box::new(left), op, right: Box::new(right) }, ty, span: b.span })
            }
            ast::Expr::Unary(u) => {
                let expr = self.lower_expr(*u.expr)?;
                let op = match u.op.kind {
                    ast::UnaryOpKind::Neg => HirUnaryOp::Neg,
                    ast::UnaryOpKind::Not => HirUnaryOp::Not,
                };

                let ty = self.check_unary_op(op, &expr, u.span)?;
                Ok(HirExpr { kind: HirExprKind::Unary { op, expr: Box::new(expr) }, ty, span: u.span })
            }
            ast::Expr::Call(c) => {
                let func = self.lower_expr(*c.func)?;
                let mut args = Vec::new();
                for a in c.args {
                    args.push(self.lower_expr(a)?);
                }
                let ty = HirType::Unknown;
                Ok(HirExpr { kind: HirExprKind::Call { func: Box::new(func), args }, ty, span: c.span })
            }
        }
    }

    fn check_binary_op(&self, left: &HirExpr, op: HirBinaryOp, right: &HirExpr, span: Span) -> Result<HirType> {
        match op {
            HirBinaryOp::Add | HirBinaryOp::Sub | HirBinaryOp::Mul | HirBinaryOp::Div | HirBinaryOp::Mod => {
                if left.ty == right.ty
                    && matches!(left.ty, HirType::Primitive(PrimitiveType::I32 | PrimitiveType::I64 | PrimitiveType::F32 | PrimitiveType::F64 | PrimitiveType::D128))
                {
                    Ok(left.ty.clone())
                }
                else {
                    Err(KqlError::semantic(
                        span,
                        format!("Cannot apply arithmetic operator {:?} to {:?} and {:?}", op, left.ty, right.ty),
                    ))
                }
            }
            HirBinaryOp::Eq | HirBinaryOp::NotEq => {
                if left.ty == right.ty {
                    Ok(HirType::Primitive(PrimitiveType::Bool))
                }
                else {
                    Err(KqlError::semantic(span, format!("Cannot compare {:?} and {:?}", left.ty, right.ty)))
                }
            }
            HirBinaryOp::Gt | HirBinaryOp::Lt | HirBinaryOp::GtEq | HirBinaryOp::LtEq => {
                if left.ty == right.ty
                    && matches!(
                        left.ty,
                        HirType::Primitive(PrimitiveType::I32 | PrimitiveType::I64 | PrimitiveType::F32 | PrimitiveType::F64 | PrimitiveType::DateTime)
                    )
                {
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
}
