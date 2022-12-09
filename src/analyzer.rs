use std::collections::{HashMap, HashSet};

use crate::{ast::*, error::Error};

pub struct Analyzer<'a>{
    ast: &'a Vec<Expr>,
    source: String,
    filename: String,
    variables: Vec<HashSet<String>>,
    errors: Vec<Error>,
}
impl <'a> Analyzer<'a> {
    pub fn new(ast: &'a Vec<Expr>, source: String, filename: String,) -> Self {
        Self {
            ast,
            variables: vec![HashSet::new()],
            source,
            filename,
            errors: Vec::new(),
        }
    }
    pub fn analyze(&mut self) -> bool{
        for expr in self.ast.clone() {
            self.handle(&expr);
        }
        self.has_errors()
    }
    pub fn handle(&mut self, expr: &Expr) {
        match &expr.inner {
            ExprKind::Binary(lhs,_,rhs) => {
                self.handle(lhs);
                self.handle(rhs);
            }
            ExprKind::Var(n, val, _) => {
                self.handle(val);
              
                self.variables.last_mut().unwrap().insert(n.to_string());
            }
            ExprKind::Block(exprs) => {
                self.start_scope();
                for expr in exprs {
                    self.handle(expr);
                }
                self.end_scope();
            }
            ExprKind::Ident(ident) => {
                match self.get(ident) {
                    Some(_) => {}
                    None => {
                        self.errors.push(Error {
                            source: self.source.clone(),
                            file_name: self.filename.clone(),
                            message: "Variable not found in the current scope".to_owned(),
                            span: expr.span,
                            help: "".to_owned(),
                        })
                    }
                }
            }
            _ => {}
        }
    }
    fn get(&self, ident: &String) -> Option<String>{
        self.variables.iter().rev().find_map(|map| map.get(ident)).cloned()
    }
    fn start_scope(&mut self) {
        self.variables.push(HashSet::new());
    }
    fn end_scope(&mut self) {
        self.variables.pop();
    }
    pub fn has_errors(&self) -> bool {
        return !self.errors.is_empty()
    }
    pub fn finish(&self) {
        for err in self.errors.clone() {
            err.show();
        }
    }
}