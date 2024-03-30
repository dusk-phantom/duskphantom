use crate::{errors::MiddelError, frontend, utils::mem::ObjPtr};
use ir::ir_builder::IRBuilder;

mod analysis;
pub mod ir;
mod transform;

use std::pin::Pin;
pub struct Program {
    pub module: ir::Module,
    pub mem_pool: Pin<Box<IRBuilder>>,
}

pub fn gen(program: &frontend::Program) -> Result<Program, MiddelError> {
    let mut result = Program::new();
    for decl in program.module.iter() {
        gen_decl(decl, &mut result);
    }
    Ok(result)
}

fn unique_debug(base: &'static str) -> String {
    base.to_string()
}

fn unique_name() -> String {
    "".to_string()
}

fn gen_type(ty: &frontend::Type) -> ir::ValueType {
    match ty {
        frontend::Type::Void => ir::ValueType::Void,
        frontend::Type::Int32 => ir::ValueType::Int,
        frontend::Type::Float32 => ir::ValueType::Float,
        frontend::Type::String => todo!(),
        frontend::Type::Char => todo!(),
        frontend::Type::Boolean => ir::ValueType::Bool,
        frontend::Type::Pointer(ty) => ir::ValueType::Pointer(Box::new(gen_type(ty))),
        frontend::Type::Array(ty, n) => ir::ValueType::Array(Box::new(gen_type(ty)), *n),
        frontend::Type::Function(_, _) => todo!(),
        frontend::Type::Enum(_) => todo!(),
        frontend::Type::Union(_) => todo!(),
        frontend::Type::Struct(_) => todo!(),
    }
}

fn gen_decl(decl: &frontend::Decl, program: &mut Program) -> Option<MiddelError> {
    match decl {
        frontend::Decl::Var(_, _, _) => todo!("global is not supported"),
        frontend::Decl::Func(ty, id, op) => {
            if let (Some(stmt), frontend::Type::Function(return_ty, params)) = (op, ty) {
                let mut fptr = program
                    .mem_pool
                    .new_function(id.clone(), gen_type(return_ty));
                for param in params.iter() {
                    fptr.params.push(ir::Parameter {
                        name: param.id.clone()?,
                        value_type: gen_type(&param.ty),
                    })
                }
                let (entry, exit) = gen_stmt(stmt, program);
                fptr.entry = Some(entry);
                fptr.exit = Some(exit);
                None
            } else {
                None
            }
        }
        frontend::Decl::Enum(_, _) => todo!(),
        frontend::Decl::Union(_, _) => todo!(),
        frontend::Decl::Struct(_, _) => todo!(),
    }
}

fn gen_expr(
    expr: &frontend::Expr,
    program: &mut Program,
    ty: &ir::ValueType,
) -> (ir::InstPtr, ir::BBPtr, ir::BBPtr) {
    match expr {
        frontend::Expr::Var(_) => todo!(),
        frontend::Expr::Pack(_) => todo!(),
        frontend::Expr::Map(_) => todo!(),
        frontend::Expr::Index(_, _) => todo!(),
        frontend::Expr::Field(_, _) => todo!(),
        frontend::Expr::Select(_, _) => todo!(),
        frontend::Expr::Int32(_) => todo!(),
        frontend::Expr::Float32(_) => todo!(),
        frontend::Expr::String(_) => todo!(),
        frontend::Expr::Char(_) => todo!(),
        frontend::Expr::Bool(_) => todo!(),
        frontend::Expr::Call(_, _) => todo!(),
        frontend::Expr::Unary(_, _) => todo!(),
        frontend::Expr::Binary(_, _, _) => todo!(),
        frontend::Expr::Conditional(_, _, _) => todo!(),
    }
}

fn gen_inner_decl(decl: &frontend::Decl, program: &mut Program) -> (ir::BBPtr, ir::BBPtr) {
    match decl {
        frontend::Decl::Var(ty, _, op) => {
            if let Some(expr) = op {
                let mty = gen_type(ty);
                let (expr_ptr, entry, mut exit) = gen_expr(expr, program, &mty);
                let load = program
                    .mem_pool
                    .get_load(mty.clone(), ir::Operand::Instruction(expr_ptr));
                let alloca = program.mem_pool.get_alloca(mty.clone(), 1);
                let store = program.mem_pool.get_store(
                    ir::Operand::Instruction(load),
                    ir::Operand::Instruction(alloca),
                );
                exit.push_back(load);
                exit.push_back(alloca);
                exit.push_back(store);
                (entry, exit)
            } else {
                let bb = program.mem_pool.new_basicblock(unique_debug("nothing"));
                (bb, bb)
            }
        }
        frontend::Decl::Func(_, _, _) => todo!(),
        frontend::Decl::Enum(_, _) => todo!(),
        frontend::Decl::Union(_, _) => todo!(),
        frontend::Decl::Struct(_, _) => todo!(),
    }
}

fn gen_stmt(stmt: &frontend::Stmt, program: &mut Program) -> (ir::BBPtr, ir::BBPtr) {
    match stmt {
        frontend::Stmt::Nothing => {
            let bb = program.mem_pool.new_basicblock(unique_debug("nothing"));
            (bb, bb)
        }
        frontend::Stmt::Decl(decl) => todo!(),
        frontend::Stmt::Expr(_) => todo!(),
        frontend::Stmt::If(_, _, _) => todo!(),
        frontend::Stmt::While(_, _) => todo!(),
        frontend::Stmt::DoWhile(_, _) => todo!(),
        frontend::Stmt::For(_, _, _, _) => todo!(),
        frontend::Stmt::Break => todo!(),
        frontend::Stmt::Continue => todo!(),
        frontend::Stmt::Return(_) => todo!(),
        frontend::Stmt::Block(_) => todo!(),
    }
}

pub fn optimize(program: &mut Program) {
    todo!()
}

impl Program {
    pub fn new() -> Self {
        let program_mem_pool = Box::pin(IRBuilder::new());
        let mem_pool: ObjPtr<IRBuilder> = ObjPtr::new(&program_mem_pool);
        Self {
            mem_pool: program_mem_pool,
            module: ir::Module::new(mem_pool),
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        self.mem_pool.clear();
    }
}
