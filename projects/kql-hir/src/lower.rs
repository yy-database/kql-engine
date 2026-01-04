use crate::*;
use kql_ast as ast;
use kql_types::{KqlError, Result};

pub struct Lowerer {
    pub db: HirDatabase,
}

impl Lowerer {
    pub fn new() -> Self {
        Self { db: HirDatabase::default() }
    }

    pub fn lower_decls(&mut self, decls: Vec<ast::Decl>) -> Result<()> {
        // First pass: Collect all names to allow forward references
        for decl in &decls {
            let name = match decl {
                ast::Decl::Struct(s) => &s.name.name,
                ast::Decl::Enum(e) => &e.name.name,
                ast::Decl::Let(l) => &l.name.name,
            };
            let id = self.db.alloc_id();
            self.db.name_to_id.insert(name.clone(), id);
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
        let mut fields = Vec::new();
        for f in s.fields {
            fields.push(HirField { name: f.name.name, ty: self.lower_type(f.ty)?, span: f.span });
        }
        Ok(HirStruct { id, name: s.name.name, fields, span: s.span })
    }

    fn lower_enum(&mut self, e: ast::EnumDecl) -> Result<HirEnum> {
        let id = *self.db.name_to_id.get(&e.name.name).unwrap();
        let mut variants = Vec::new();
        for v in e.variants {
            let fields = if let Some(f_vec) = v.fields {
                let mut hir_f_vec = Vec::new();
                for f in f_vec {
                    hir_f_vec.push(HirField { name: f.name.name, ty: self.lower_type(f.ty)?, span: f.span });
                }
                Some(hir_f_vec)
            }
            else {
                None
            };
            variants.push(HirVariant { name: v.name.name, fields, span: v.span });
        }
        Ok(HirEnum { id, name: e.name.name, variants, span: e.span })
    }

    fn lower_let(&mut self, l: ast::LetDecl) -> Result<HirLet> {
        let id = *self.db.name_to_id.get(&l.name.name).unwrap();
        let value = self.lower_expr(l.value)?;
        let ty = if let Some(ast_ty) = l.ty { self.lower_type(ast_ty)? } else { value.ty.clone() };
        Ok(HirLet { id, name: l.name.name, ty, value, span: l.span })
    }

    fn lower_type(&mut self, ty: ast::Type) -> Result<HirType> {
        match ty {
            ast::Type::Named(n) => {
                match n.name.as_str() {
                    "i32" | "i64" => Ok(HirType::Primitive(PrimitiveType::Int)),
                    "f32" | "f64" => Ok(HirType::Primitive(PrimitiveType::Float)),
                    "String" => Ok(HirType::Primitive(PrimitiveType::String)),
                    "Bool" => Ok(HirType::Primitive(PrimitiveType::Bool)),
                    _ => {
                        if let Some(&id) = self.db.name_to_id.get(&n.name) {
                            // Check if it's a struct or enum
                            // For now, assume if it exists in name_to_id, it's a valid type
                            Ok(HirType::Struct(id)) // Simplified: should distinguish between struct/enum
                        }
                        else {
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
                        (HirExprKind::Literal(HirLiteral::Int(n.parse().unwrap_or(0))), HirType::Primitive(PrimitiveType::Int))
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
                    // Find type of the variable
                    let ty = if let Some(l) = self.db.lets.get(&id) { l.ty.clone() } else { HirType::Unknown };
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
                // Basic type inference: use left type
                let ty = left.ty.clone();
                Ok(HirExpr { kind: HirExprKind::Binary { left: Box::new(left), op, right: Box::new(right) }, ty, span: b.span })
            }
            ast::Expr::Unary(u) => {
                let expr = self.lower_expr(*u.expr)?;
                let op = match u.op.kind {
                    ast::UnaryOpKind::Neg => HirUnaryOp::Neg,
                    ast::UnaryOpKind::Not => HirUnaryOp::Not,
                };
                let ty = expr.ty.clone();
                Ok(HirExpr { kind: HirExprKind::Unary { op, expr: Box::new(expr) }, ty, span: u.span })
            }
            ast::Expr::Call(c) => {
                let func = self.lower_expr(*c.func)?;
                let mut args = Vec::new();
                for a in c.args {
                    args.push(self.lower_expr(a)?);
                }
                // Call return type: Unknown for now
                let ty = HirType::Unknown;
                Ok(HirExpr { kind: HirExprKind::Call { func: Box::new(func), args }, ty, span: c.span })
            }
        }
    }
}
