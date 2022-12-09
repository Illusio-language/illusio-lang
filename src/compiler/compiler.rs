use cranelift::prelude::{settings::Flags, *};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataContext, FuncId, Linkage, Module};
use std::collections::HashMap;

use super::ir::*;
pub struct Compiler {
    module: JITModule,
    builder_ctx: FunctionBuilderContext,
    ctx: codegen::Context,
}
impl Compiler {
    pub fn new() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let module = JITModule::new(builder.unwrap());
        Self {
            builder_ctx: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
        }
    }
    pub fn compile(&mut self, stmts: Vec<Stmt>) -> Result<*const u8, String> {
        self.translate(stmts)?;
        let id = self
            .module
            .declare_function("main", Linkage::Export, &self.ctx.func.signature)
            .map_err(|err| err.to_string())?;
        self.module
            .define_function(id, &mut self.ctx)
            .map_err(|e| e.to_string())?;
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions();
        let code = self.module.get_finalized_function(id);
        Ok(code)
    }
    pub fn translate(&mut self, stmts: Vec<Stmt>) -> Result<(), String> {
        self.ctx
            .func
            .signature
            .returns
            .push(AbiParam::new(types::I64));
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_ctx);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        let mut handler = Handler {
            builder,
            variables: HashMap::new(),
            data: DataContext::new(),
            module: &mut self.module,
        };
        for stmt in stmts {
            handler.translate_stmt(stmt);
        }
        let zero = handler.builder.ins().iconst(types::I64, 0);
        handler.builder.ins().return_(&[zero]);
        handler.builder.finalize();
        println!("{}", self.ctx.func);
        if let Err(e) = codegen::verify_function(
            &self.ctx.func,
            &Flags::new(cranelift::codegen::settings::builder()),
        ) {
            println!("codegen error");

            let mut s = String::new();

            s.clear();

            codegen::write_function(&mut s, &self.ctx.func).unwrap();
            println!("CLIF:\n{}", s);

            panic!("errors: {:#?}", e);
        }
        Ok(())
    }
}
pub struct Handler<'a> {
    builder: FunctionBuilder<'a>,
    data: DataContext,
    module: &'a mut JITModule,
    variables: HashMap<String, Variable>,
}
impl<'a> Handler<'a> {
    fn translate_stmt(&mut self, stmt: Stmt) -> Value {
        match stmt {
            Stmt::ExprStmt(expr) => {
                self.translate_expr(expr);
                self.null()
            }
            Stmt::If(condition, then_body, else_body) => {
                let condition_value = self.translate_expr(condition);

                let then_block = self.builder.create_block();
                let else_block = self.builder.create_block();
                let merge_block = self.builder.create_block();

                // If-else constructs in the toy language have a return value.
                // In traditional SSA form, this would produce a PHI between
                // the then and else bodies. Cranelift uses block parameters,
                // so set up a parameter in the merge block, and we'll pass
                // the return values to it from the branches.
                self.builder.append_block_param(merge_block, types::I64);

                // Test the if condition and conditionally branch.
                self.builder.ins().brz(condition_value, else_block, &[]);
                // Fall through to then block.
                self.builder.ins().jump(then_block, &[]);

                self.builder.switch_to_block(then_block);
                self.builder.seal_block(then_block);
                let mut then_return = self.builder.ins().iconst(types::I64, 0);
                for stmt in then_body {
                    then_return = self.translate_stmt(stmt);
                }

                // Jump to the merge block, passing it the block return value.
                self.builder.ins().jump(merge_block, &[then_return]);

                self.builder.switch_to_block(else_block);
                self.builder.seal_block(else_block);
                let mut else_return = self.builder.ins().iconst(types::I64, 0);
                for stmt in else_body {
                    else_return = self.translate_stmt(stmt);
                }

                // Jump to the merge block, passing it the block return value.
                self.builder.ins().jump(merge_block, &[else_return]);

                // Switch to the merge block for subsequent statements.
                self.builder.switch_to_block(merge_block);

                // We've now seen all the predecessors of the merge block.
                self.builder.seal_block(merge_block);
                let phi = self.builder.block_params(merge_block)[0];
                phi
            }
            Stmt::SetVar(name, expr) => {
                let val = self.translate_expr(expr);
                let var = self.variables.get(&name).expect("Variable not found");
                self.builder.def_var(*var, val);
                self.null()
            }
            Stmt::Var(name, expr, var_type) => {
                let index: usize = self.variables.len();
                let var: Variable = Variable::new(index);
                let val: Value = self.translate_expr(expr);
                let ty = self.to_cranelift_ty(&var_type);
                self.variables.insert(name, var);
                self.builder.declare_var(var, ty);
                self.builder.def_var(var, val);
                self.null()
            }
        }
    }
    fn null(&mut self) -> Value {
        self.builder.ins().iconst(types::I64, 0)
    }
    fn translate_expr(&mut self, expr: Expr) -> Value {
        use Expr::*;
        match expr {
            GetVar(name) => {
                let var = self.variables.get(&name).expect("Variable not found");
                self.builder.use_var(*var)
            }
            Value(val, ir_ty) => {
                let ty = self.to_cranelift_ty(&ir_ty);
                match val {
                    IrValue::Literal(lit) => match lit {
                        Literal::Int(i) => self.builder.ins().iconst(ty, i),
                        Literal::Float(f) => self.builder.ins().f64const(f),
                        Literal::Str(s) => {
                            let mut string = s;
                            string.push('\0');
                            let boxed_s = string.into_bytes().into_boxed_slice();
                            let id = self.module.declare_anonymous_data(false, false).unwrap();

                            self.data.define(boxed_s);
                            self.module.define_data(id, &self.data).unwrap();
                            self.data.clear();
                            let value =
                                self.module.declare_data_in_func(id, &mut self.builder.func);

                            self.builder.ins().global_value(types::I64, value)
                        }
                        Literal::Bool(b) => self.builder.ins().bconst(ty, b),
                    },
                    IrValue::Binary(left, op, right) => {
                        let left = self.translate_expr(*left);
                        let right = self.translate_expr(*right);
                        match op {
                            BinOp::Add => {
                                if ty.is_float() {
                                    self.builder.ins().fadd(left, right)
                                } else {
                                    self.builder.ins().iadd(left, right)
                                }
                            }
                            BinOp::Sub => {
                                if ty.is_float() {
                                    self.builder.ins().fsub(left, right)
                                } else {
                                    self.builder.ins().isub(left, right)
                                }
                            }
                            BinOp::Mul => {
                                if ty.is_float() {
                                    self.builder.ins().fmul(left, right)
                                } else {
                                    self.builder.ins().imul(left, right)
                                }
                            }
                            BinOp::Div => {
                                if ty.is_float() {
                                    self.builder.ins().fdiv(left, right)
                                } else {
                                    self.builder.ins().udiv(left, right)
                                }
                            }
                            BinOp::Eq => {
                                if ty.is_float() {
                                    self.builder.ins().fcmp(FloatCC::Equal, left, right)
                                } else {
                                    self.builder.ins().icmp(IntCC::Equal, left, right)
                                }
                            }
                            BinOp::Neq => {
                                if ty.is_float() {
                                    self.builder.ins().fcmp(FloatCC::NotEqual, left, right)
                                } else {
                                    self.builder.ins().icmp(IntCC::NotEqual, left, right)
                                }
                            }
                            BinOp::Gt => {
                                if ty.is_float() {
                                    self.builder.ins().fcmp(FloatCC::GreaterThan, left, right)
                                } else {
                                    self.builder
                                        .ins()
                                        .icmp(IntCC::SignedGreaterThan, left, right)
                                }
                            }
                            BinOp::Lt => {
                                if ty.is_float() {
                                    self.builder.ins().fcmp(FloatCC::LessThan, left, right)
                                } else {
                                    self.builder.ins().icmp(IntCC::SignedLessThan, left, right)
                                }
                            }
                            BinOp::Ge => {
                                if ty.is_float() {
                                    self.builder.ins().fcmp(
                                        FloatCC::GreaterThanOrEqual,
                                        left,
                                        right,
                                    )
                                } else {
                                    self.builder.ins().icmp(
                                        IntCC::SignedGreaterThanOrEqual,
                                        left,
                                        right,
                                    )
                                }
                            }
                            BinOp::Le => {
                                if ty.is_float() {
                                    self.builder
                                        .ins()
                                        .fcmp(FloatCC::LessThanOrEqual, left, right)
                                } else {
                                    self.builder.ins().icmp(
                                        IntCC::SignedLessThanOrEqual,
                                        left,
                                        right,
                                    )
                                }
                            }
                        }
                    }
                }
            }
            PrintStr(arg) => {
                let args = &[self.translate_expr(*arg)];
                let mut sig = self.module.make_signature();
                sig.params.push(AbiParam::new(types::I64));
                sig.returns.push(AbiParam::new(types::I64));
                let callee = self
                    .module
                    .declare_function("puts", Linkage::Import, &sig)
                    .unwrap();
                let local_callee = self.module.declare_func_in_func(callee, self.builder.func);
                let call = self.builder.ins().call(local_callee, args);
                self.builder.inst_results(call)[0]
            }
            PrintInt(arg) => {
                let args = &[
                    self.translate_expr(Expr::Value(
                        IrValue::Literal(Literal::Str("%d\n".to_owned())),
                        IrType::Str,
                    )),
                    self.translate_expr(*arg),
                ];
                let mut sig = self.module.make_signature();
                sig.params.push(AbiParam::new(types::I64));
                sig.params.push(AbiParam::new(types::I64));
                sig.returns.push(AbiParam::new(types::I64));
                let callee = self
                    .module
                    .declare_function("printf", Linkage::Import, &sig)
                    .unwrap();
                let local_callee = self.module.declare_func_in_func(callee, self.builder.func);
                let call = self.builder.ins().call(local_callee, args);
                self.builder.inst_results(call)[0]
            }
            PrintFloat(arg) => {
                let args = &[
                    self.translate_expr(Expr::Value(
                        IrValue::Literal(Literal::Str("%f\n".to_owned())),
                        IrType::Str,
                    )),
                    self.translate_expr(*arg),
                ];
                let mut sig = self.module.make_signature();
                sig.params.push(AbiParam::new(types::I64));
                sig.params.push(AbiParam::new(types::F64));
                sig.returns.push(AbiParam::new(types::I64));
                let callee = self
                    .module
                    .declare_function("printf", Linkage::Import, &sig)
                    .unwrap();
                let local_callee = self.module.declare_func_in_func(callee, self.builder.func);
                let call = self.builder.ins().call(local_callee, args);
                self.builder.inst_results(call)[0]
            }
        }
    }
    fn to_cranelift_ty(&self, ir_ty: &IrType) -> Type {
        match ir_ty {
            IrType::Int => types::I64,
            IrType::Bool => types::B1,
            IrType::Float => types::F64,
            IrType::Str => types::I64,
        }
    }
}
