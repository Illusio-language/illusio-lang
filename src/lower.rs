use crate::ast::*;
use crate::compiler::builder::Builder;
use crate::compiler::ir::{self, IrType};
use crate::token::TokenKind;
use crate::typechecker::TypeChecker;
pub struct Lower<'a> {
    ast: &'a Vec<Expr>,
    builder: Builder,
    checker: TypeChecker<'a>,
}
impl<'a> Lower<'a> {
    pub fn new(ast: &'a Vec<Expr>, checker: TypeChecker<'a>) -> Self {
        Self {
            ast,
            builder: Builder::new(),
            checker,
        }
    }
    pub fn finish(&self) {
        self.builder.finish()
    }
    pub fn translate(&mut self) {
        for stmt in self.ast {
            let stmt =self.translate_stmt(stmt.clone());
            self.builder.stmt(stmt);
        }
    }
    pub fn translate_stmt(&mut self, stmt: Expr) -> ir::Stmt {
        match stmt.inner {
            ExprKind::Var(name, value, ty) => {
                let ir_ty = self.to_ir_type(ty);
                let value = self.translate_expr(*value);
                self.builder.new_var(&name, value, ir_ty)
            }
            ExprKind::If(if_,then) => {
                let condition = self.translate_expr(*if_);
                let mut block = Vec::new();
                for s in then {
                    let s = self.translate_stmt(s);
                    block.push(s);
                }
                self.builder.if_(condition, block, vec![])
            }
            _ => {
                let expr = self.translate_expr(stmt);
                let stmt = self.builder.expr(expr);
                stmt
            }
        }
    }
    pub fn translate_expr(&mut self, expr: Expr) -> ir::Expr {
        match expr.inner {
            ExprKind::Float(f) => self.builder.float(f),
            ExprKind::Ident(ident) => self.builder.get_var(&ident),
            ExprKind::Str(s) => self.builder.string(&s),
            ExprKind::Int(i) => self.builder.int(i) ,
            ExprKind::Binary(left, op, right) => {
                let op = match op {
                    TokenKind::Plus => ir::BinOp::Add,
                    TokenKind::Minus => ir::BinOp::Sub,
                    TokenKind::Asterisk => ir::BinOp::Mul,
                    TokenKind::Slash => ir::BinOp::Div,
                    TokenKind::EqualTo => ir::BinOp::Eq,
                    TokenKind::NotEqual => ir::BinOp::Neq,
                    TokenKind::GreaterThan => ir::BinOp::Gt,
                    TokenKind::LessThan => ir::BinOp::Lt,
                    TokenKind::GreaterThanEqual => ir::BinOp::Ge,
                    TokenKind::LessThanEqual => ir::BinOp::Le,
                    _ => todo!(),
                };
                let ty = self.checker.handle(&left);
                let left = self.translate_expr(*left);
                let right = self.translate_expr(*right);
                let ir_type = self.to_ir_type(ty);
                self.builder.binary(left, op, right, ir_type)
            }
            ExprKind::Bool(b) => self.builder.boolean(b),
            ExprKind::Puts(s) =>{
                let s = self.translate_expr(s.get(0).unwrap().clone());
                self.builder.str_print(s)
            }
            _ => todo!(),
        }
    }
    fn to_ir_type(&self, ty: Type) -> IrType {
        match ty {
            Type::String => IrType::Str,
            Type::Int => IrType::Int,
            Type::Float => IrType::Float,
            Type::Bool => IrType::Bool,
            _ => todo!(),
        }
    }
}
