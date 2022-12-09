#[derive(Debug, Clone)]

pub enum BinOp {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Eq, // =
    Neq,// !=
    Gt, // >
    Lt, // <
    Ge, // >=
    Le, // <=
}
impl BinOp {
    pub fn inverse(self) -> Self {
        use BinOp::*;
        match self {
            Eq => Neq,
            Gt => Lt,

            _ => panic!("Cant inverse")
        }
    }
}
#[derive(Debug, Clone)]

pub enum Expr {
    Value(IrValue, IrType),
    GetVar(String),
    PrintStr(Box<Expr>),
    PrintInt(Box<Expr>),
    PrintFloat(Box<Expr>),
}
#[derive(Debug, Clone)]

pub enum IrValue {
    Literal(Literal),
    Binary(Box<Expr>, BinOp, Box<Expr>),
}
#[derive(Debug, Clone)]

pub enum Literal {
    Bool(bool),
    Str(String),
    Int(i64),
    Float(f64),
}
#[derive(Debug, Clone)]

pub enum IrType {
    Int,
    Float,
    Str,
    Bool,
}
#[derive(Debug, Clone)]
pub enum Stmt {
    ExprStmt(Expr),
    SetVar(String, Expr),
    Var(String, Expr, IrType),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
}
impl Expr {
    pub fn boxed(self) -> Box<Self> {
        return Box::new(self)
    }
}