use std::{collections::HashMap, hash::Hash};

use crate::{
    ast::{Expr, ExprKind, Type},
    error::Error,
    traits::Item,
};
pub struct FuncSig {
    return_type: Type,
    params_type: Vec<Type>,
}
pub struct TypeChecker<'a> {
    ast: &'a Vec<Expr>,
    errors: Vec<Error>,
    sigs: HashMap<String, FuncSig>,
    source: String,
    filename: String,
}
impl<'a> TypeChecker<'a> {
    pub fn new(ast: &'a Vec<Expr>, source: &str, filename: &str) -> Self {
        Self {
            ast,
            source: source.to_owned(),
            filename: filename.to_owned(),
            errors: Vec::new(),
            sigs: HashMap::new(),
        }
    }
    pub fn check(&mut self) -> bool {
        for expr in self.ast.clone() {
            self.handle(&expr);
        }
        self.had_errors()
    }
    pub fn handle(&mut self, expr: &Expr) -> Type {
        match &expr.inner {
            ExprKind::Int(_) => Type::Int,
            ExprKind::Float(_) => Type::Float,
            ExprKind::Ident(_) => {
                Type::Int
            }
            ExprKind::Str(_) => Type::String,
            ExprKind::Var(_, val, ty) => {
                let ty_val = self.handle(&val);
                if &ty_val != ty {
                    self.errors.push(Error {
                        source: self.source.clone(),
                        file_name: self.filename.clone(),
                        message: format!(
                            "Expected type {} found type {}",
                            format!("{:?}", ty).to_lowercase(),
                            format!("{:?}", ty_val).to_lowercase()
                        ),
                        span: val.span,
                        help: "".to_owned(),
                    })
                }
                Type::None
            }
            ExprKind::Binary(lhs, _, rhs) => {
                let lhs_ty = self.handle(&lhs);
                let rhs_ty = self.handle(&rhs);
                if lhs_ty != rhs_ty {
                    self.errors.push(Error {
                        source: self.source.clone(),
                        file_name: self.filename.clone(),
                        message: "Cannot do binary operations, with different types".to_string(),
                        span: expr.span,
                        help: "".to_owned(),
                    })
                }
                // We could return lhs or rhs, doesn't matter
                lhs_ty
            }
            ExprKind::Unary(_, _) => todo!(),
            ExprKind::Ref(_) => {
                let t = self.handle(expr);
                Type::Ptr(t.boxed())
            }
            ExprKind::If(condition, exprs) => {
                self.handle(&condition);
                for expr in exprs {
                    self.handle(expr);
                }
                Type::None
            }
            ExprKind::Enum(_, _) => Type::None,
            ExprKind::FunctionCall(_, _) => todo!(),
            ExprKind::FunctionDeclaration(name, params, return_type, exprs) => {
                let mut params_type = Vec::new();
                for param in params {
                    params_type.push(param.param_type.clone());
                }
                self.sigs.insert(name.to_owned(), FuncSig { return_type: return_type.clone(), params_type: params_type });
                Type::None
            }
            ExprKind::Bool(_) => Type::Bool,
            ExprKind::Block(exprs) => {
                for expr in exprs {
                    self.handle(expr);
                }
                Type::None
            }
            ExprKind::Error => Type::None,
            ExprKind::Eof => Type::None,
            ExprKind::Puts(_) => Type::None
        }
    }
    pub fn finish(&self) {
        for err in &self.errors {
            err.show();
        }
    }

    pub fn had_errors(&self) -> bool {
        return !self.errors.is_empty();
    }
}
