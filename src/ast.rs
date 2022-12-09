use crate::token::{Span, TokenKind};
use crate::traits::*;
use std::fmt::Display;
#[derive(PartialEq, Debug, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}
impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}
#[derive(PartialEq, Debug, Clone)]
pub enum ExprKind {
    Float(f64),
    Ident(String),
    Str(String),
    Var(String, Box<Expr>, Type),
    Int(i64),
    Binary(Box<Expr>, TokenKind, Box<Expr>),
    Unary(TokenKind, Box<Expr>),
    Ref(Box<Expr>),
    If(Box<Expr>, Vec<Expr>),
    Enum(String,Vec<String>),
    FunctionCall(String, Vec<Expr>), 
    FunctionDeclaration(String, Vec<Param>,Type,Vec<Expr>),
    Bool(bool),
    Block(Vec<Expr>),
    Puts(Vec<Expr>),
    Error,
    Eof,
}
impl<'a> Item for ExprKind {
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
    fn show(&self) {
        println!("{:?}", self)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    String,
    Int,
    Float,
    Bool,
    Ptr(Box<Type>),
    None,
}
impl Item for Type {
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
    fn show(&self) {
        println!("{:?}", self)
    }
}
pub type Expr = Spanned<ExprKind>;

impl<'a> Item for Expr {
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
    fn show(&self) {
        println!("{:?}", self.inner);
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub param_type: Type,
    pub name: String,
}