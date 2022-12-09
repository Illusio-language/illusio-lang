use super::{ir::{Stmt, Expr, IrType, Literal, IrValue, BinOp}, compiler::Compiler};

pub struct Builder {
    pub code: Vec<Stmt>
}
impl Builder {
    pub fn new() -> Self {
        Builder { code: Vec::new() }
    }
    pub fn expr(&self, expr: Expr) -> Stmt{
        Stmt::ExprStmt(expr)
    }
    pub fn stmt(&mut self, stmt: Stmt) {
        self.code.push(stmt);
    }
    pub fn int(&self, int: i64) -> Expr {
        Expr::Value(IrValue::Literal(Literal::Int(int)), IrType::Int)
    }
    pub fn float(&self, float: f64) -> Expr {
        Expr::Value(IrValue::Literal(Literal::Float(float)), IrType::Float)
    }
    pub fn string(&self, string: &str) -> Expr {
        Expr::Value(IrValue::Literal(Literal::Str(string.to_owned())), IrType::Str)
    }
    pub fn boolean(&self, boolean: bool) -> Expr {
        Expr::Value(IrValue::Literal(Literal::Bool(boolean)), IrType::Bool)
    }
    pub fn binary(&self, left: Expr, op: BinOp, right: Expr, ty: IrType) -> Expr {
        Expr::Value(IrValue::Binary(left.boxed(), op, right.boxed()), ty)
    }
    pub fn str_print(&self, s: Expr) -> Expr {
        Expr::PrintStr(s.boxed())
    }
    pub fn int_print(&self, s: Expr) -> Expr {
        Expr::PrintInt(s.boxed())
    }
    pub fn float_print(&self, s: Expr) -> Expr {
        Expr::PrintFloat(s.boxed())
    }
    pub fn new_var(&mut self, name: &str, value: Expr, ty: IrType)-> Stmt{
        Stmt::Var(name.to_owned(), value, ty)
    }
    pub fn get_var(&self, name: &str) -> Expr {
        Expr::GetVar(name.to_owned())
    }
    pub fn set_var(&mut self, name: &str, value: Expr) {
        self.code.push(Stmt::SetVar(name.to_owned(), value));
    }
    pub fn if_(&self, condition: Expr, stmts: Vec<Stmt>, else_: Vec<Stmt>,) -> Stmt{
        Stmt::If(condition, stmts, else_)
    }
    pub fn finish(&self) {
        let mut compiler = Compiler::new();
        let code = compiler.compile(self.code.clone());
        match code {
            Ok(ptr) => {
                let run_code = unsafe { std::mem::transmute::<_, fn() -> i64>(ptr) };
                run_code();
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}