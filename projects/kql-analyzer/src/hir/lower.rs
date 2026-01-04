use super::*;
use kql_ast as ast;
use kql_types::{KqlError, Result, Span};

pub struct Lowerer {
    pub db: HirDatabase,
}

impl Lowerer {
    pub fn new() -> Self {
        Self { db: HirDatabase::default() }
    }

    pub fn lower_database(&mut self, ast_db: &ast::Database) -> Result<HirDatabase> {
        self.lower_decls(ast_db.decls.clone())?;
        Ok(self.db.clone())
    }

    pub fn lower_decls(&mut self, decls: Vec<ast::Decl>) -> Result<()> {
        // First pass: Collect all names to allow forward references
        for decl in &decls {
            let (name, kind) = match decl {
                ast::Decl::Struct(s) => (&s.name.name, HirKind::Struct),
                ast::Decl::Enum(e) => (&e.name.name, HirKind::Enum),
                ast::Decl::Let(l) => (&l.name.name, HirKind::Let),
            };
            let id = self.db.alloc_id();
            self.db.name_to_id.insert(name.clone(), id);
            self.db.id_to_kind.insert(id, kind);
        }

        // Second pass: Lower actual content
        for decl in decls {
            match decl {
                ast::Decl::Struct(s) => {
                    let hir_s = self.lower_struct(s)?;
                    self.db.structs.insert(hir_s.id, hir_s);
                }
                ast::Decl::Enum(e) => {
                    let hir_e = self.lower_enum(e)?;
                    self.db.enums.insert(hir_e.id, hir_e);
                }
                ast::Decl::Let(l) => {
                    let hir_l = self.lower_let(l)?;
                    self.db.lets.insert(hir_l.id, hir_l);
                }
            }
        }
        Ok(())
    }

    fn lower_struct(&mut self, s: ast::StructDecl) -> Result<HirStruct> {
        let id = *self.db.name_to_id.get(&s.name.name).unwrap();
        let attrs = self.lower_attrs(s.attrs)?;
        let mut fields = Vec::new();
        for f in s.fields {
            fields.push(HirField {
                attrs: self.lower_attrs(f.attrs)?,
                name: f.name.name,
                ty: self.lower_type(f.ty)?,
                span: f.span,
            });
        }
        Ok(HirStruct {
            id,
            attrs,
            name: s.name.name,
            fields,
            span: s.span,
        })
    }

    fn lower_enum(&mut self, e: ast::EnumDecl) -> Result<HirEnum> {
        let id = *self.db.name_to_id.get(&e.name.name).unwrap();
        let attrs = self.lower_attrs(e.attrs)?;
        let mut variants = Vec::new();
        for v in e.variants {
            let fields = if let Some(f_vec) = v.fields {
                let mut hir_f_vec = Vec::new();
                for f in f_vec {
                    hir_f_vec.push(HirField {
                        attrs: self.lower_attrs(f.attrs)?,
                        name: f.name.name,
                        ty: self.lower_type(f.ty)?,
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
            variants,
            span: e.span,
        })
    }

    fn lower_let(&mut self, l: ast::LetDecl) -> Result<HirLet> {
        let id = *self.db.name_to_id.get(&l.name.name).unwrap();
        let attrs = self.lower_attrs(l.attrs)?;
        let value = self.lower_expr(l.value)?;
        let ty = if let Some(ast_ty) = l.ty {
            self.lower_type(ast_ty)?
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
                    args.push(self.lower_expr(arg)?);
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

    fn lower_type(&mut self, ty: ast::Type) -> Result<HirType> {
        match ty {
            ast::Type::Named(n) => {
                // Special handling for Key<T>
                if n.name == "Key" {
                    if let Some(args) = n.args {
                        if args.len() == 1 {
                            let inner = self.lower_type(args[0].clone())?;
                            return Ok(HirType::Key(Box::new(inner)));
                        }
                    }
                    return Err(KqlError::semantic(
                        n.span,
                        "Key type must have exactly one generic argument, e.g., Key<i32>".to_string(),
                    ));
                }

                // Special handling for List<T> as an alternative to [T]
                if n.name == "List" {
                    if let Some(args) = n.args {
                        if args.len() == 1 {
                            let inner = self.lower_type(args[0].clone())?;
                            return Ok(HirType::List(Box::new(inner)));
                        }
                    }
                    return Err(KqlError::semantic(
                        n.span,
                        "List type must have exactly one generic argument, e.g., List<String>".to_string(),
                    ));
                }

                // Special handling for Option<T> as an alternative to T?
                if n.name == "Option" {
                    if let Some(args) = n.args {
                        if args.len() == 1 {
                            let inner = self.lower_type(args[0].clone())?;
                            return Ok(HirType::Optional(Box::new(inner)));
                        }
                    }
                    return Err(KqlError::semantic(
                        n.span,
                        "Option type must have exactly one generic argument, e.g., Option<String>".to_string(),
                    ));
                }

                match n.name.as_str() {
                    "i32" | "i64" => Ok(HirType::Primitive(PrimitiveType::Integer32)),
                    "f32" | "f64" => Ok(HirType::Primitive(PrimitiveType::Float32)),
                    "String" => Ok(HirType::Primitive(PrimitiveType::String)),
                    "bool" => Ok(HirType::Primitive(PrimitiveType::Bool)),
                    "date_time" => Ok(HirType::Primitive(PrimitiveType::DateTime)),
                    "uuid" => Ok(HirType::Primitive(PrimitiveType::Uuid)),
                    _ => {
                        if let Some(&id) = self.db.name_to_id.get(&n.name) {
                            match self.db.id_to_kind.get(&id) {
                                Some(HirKind::Struct) => Ok(HirType::Struct(id)),
                                Some(HirKind::Enum) => Ok(HirType::Enum(id)),
                                _ => Err(KqlError::semantic(
                                    n.span,
                                    format!("'{}' is not a valid type", n.name),
                                )),
                            }
                        } else {
                            Err(KqlError::semantic(n.span, format!("Unknown type: {}", n.name)))
                        }
                    }
                }
            }
            ast::Type::List(l) => {
                let inner = self.lower_type(*l.inner)?;
                Ok(HirType::List(Box::new(inner)))
            }
            ast::Type::Optional(o) => {
                let inner = self.lower_type(*o.inner)?;
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
                                HirExprKind::Literal(HirLiteral::Float(n.parse().unwrap_or(0.0))),
                                HirType::Primitive(PrimitiveType::Float32),
                            )
                        }
                        else {
                            (
                                HirExprKind::Literal(HirLiteral::Int(n.parse().unwrap_or(0))),
                                HirType::Primitive(PrimitiveType::Integer32),
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
                    Err(KqlError::semantic(v.span, format!("Undefined variable: {}", v.name)))
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
                    && matches!(left.ty, HirType::Primitive(PrimitiveType::Integer32 | PrimitiveType::Float32))
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
                        HirType::Primitive(PrimitiveType::Integer32 | PrimitiveType::Float32 | PrimitiveType::DateTime)
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
                if matches!(expr.ty, HirType::Primitive(PrimitiveType::Integer32 | PrimitiveType::Float32)) {
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
