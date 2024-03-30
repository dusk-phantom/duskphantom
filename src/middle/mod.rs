use crate::{errors::MiddelError, frontend, utils::mem::ObjPtr};
use ir::ir_builder::IRBuilder;

mod analysis;
pub mod ir;
mod transform;

use std::{collections::HashMap, pin::Pin};
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

fn translate_type(ty: &frontend::Type) -> ir::ValueType {
    match ty {
        frontend::Type::Void => ir::ValueType::Void,
        frontend::Type::Int32 => ir::ValueType::Int,
        frontend::Type::Float32 => ir::ValueType::Float,
        frontend::Type::String => todo!(),
        frontend::Type::Char => todo!(),
        frontend::Type::Boolean => ir::ValueType::Bool,
        frontend::Type::Pointer(ty) => ir::ValueType::Pointer(Box::new(translate_type(ty))),
        frontend::Type::Array(ty, n) => ir::ValueType::Array(Box::new(translate_type(ty)), *n),
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
                // Create function
                let mut fptr = program
                    .mem_pool
                    .new_function(id.clone(), translate_type(return_ty));

                // Fill parameters
                for param in params.iter() {
                    fptr.params.push(ir::Parameter {
                        name: param.id.clone()?,
                        value_type: translate_type(&param.ty),
                    })
                }

                // Build function
                let mut entry = program.mem_pool.new_basicblock(unique_debug("entry"));
                let exit = gen_stmt(stmt, program, &mut HashMap::new(), &mut entry);
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

fn gen_stmt(
    stmt: &frontend::Stmt,
    program: &mut Program,
    env: &mut HashMap<String, ir::Operand>,
    exit: &mut ir::BBPtr,
) -> ir::BBPtr {
    match stmt {
        frontend::Stmt::Nothing => *exit,
        frontend::Stmt::Decl(decl) => {
            // Insert created declaration to environment
            let (id, operand, exit) = gen_stml_decl(decl, program, env, exit);
            env.insert(id, operand);
            exit
        }
        frontend::Stmt::Expr(expr) => {
            // Evaluate expression but discard its result
            let (_, exit) = gen_stmt_expr(expr, program, env, exit);
            exit
        }
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

fn gen_stml_decl(
    decl: &frontend::Decl,
    program: &mut Program,
    env: &mut HashMap<String, ir::Operand>,
    exit: &mut ir::BBPtr,
) -> (String, ir::Operand, ir::BBPtr) {
    match decl {
        frontend::Decl::Var(ty, id, op) => {
            let mty = translate_type(ty);
            if let Some(expr) = op {
                // Directly generate expression
                let (operand, new_exit) = gen_stmt_expr(expr, program, env, exit);
                (id.clone(), operand, new_exit)
            } else {
                // Alloc variable
                let alloca = program.mem_pool.get_alloca(mty.clone(), 1);
                exit.push_back(alloca);
                (id.clone(), ir::Operand::Instruction(alloca), *exit)
            }
        }
        frontend::Decl::Func(_, _, _) => todo!(),
        frontend::Decl::Enum(_, _) => todo!(),
        frontend::Decl::Union(_, _) => todo!(),
        frontend::Decl::Struct(_, _) => todo!(),
    }
}

fn gen_stmt_expr(
    expr: &frontend::Expr,
    program: &mut Program,
    env: &mut HashMap<String, ir::Operand>,
    exit: &mut ir::BBPtr,
) -> (ir::Operand, ir::BBPtr) {
    match expr {
        frontend::Expr::Var(x) => (env[x].clone(), *exit),
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
        frontend::Expr::Binary(op, lhs, rhs) => {
            let (lop, mut exit) = gen_stmt_expr(lhs, program, env, exit);
            let (rop, mut exit) = gen_stmt_expr(rhs, program, env, &mut exit);
            match op {
                frontend::BinaryOp::Assign => {
                    // TODO: Get correct value type
                    // Load RHS
                    let load = program.mem_pool.get_load(ir::ValueType::Int, rop);
                    exit.push_back(load);

                    // Store in LHS
                    let store = program
                        .mem_pool
                        .get_store(ir::Operand::Instruction(load), lop.clone());
                    exit.push_back(store);
                    (lop, exit)
                }
                frontend::BinaryOp::AssignAdd => todo!(),
                frontend::BinaryOp::AssignSub => todo!(),
                frontend::BinaryOp::AssignMul => todo!(),
                frontend::BinaryOp::AssignDiv => todo!(),
                frontend::BinaryOp::AssignMod => todo!(),
                frontend::BinaryOp::AssignShr => todo!(),
                frontend::BinaryOp::AssignShl => todo!(),
                frontend::BinaryOp::AssignAnd => todo!(),
                frontend::BinaryOp::AssignOr => todo!(),
                frontend::BinaryOp::AssignXor => todo!(),
                frontend::BinaryOp::Add => todo!(),
                frontend::BinaryOp::Sub => todo!(),
                frontend::BinaryOp::Mul => todo!(),
                frontend::BinaryOp::Div => todo!(),
                frontend::BinaryOp::Mod => todo!(),
                frontend::BinaryOp::Shr => todo!(),
                frontend::BinaryOp::Shl => todo!(),
                frontend::BinaryOp::And => todo!(),
                frontend::BinaryOp::Or => todo!(),
                frontend::BinaryOp::Xor => todo!(),
                frontend::BinaryOp::Gt => todo!(),
                frontend::BinaryOp::Lt => todo!(),
                frontend::BinaryOp::Ge => todo!(),
                frontend::BinaryOp::Le => todo!(),
                frontend::BinaryOp::Eq => todo!(),
                frontend::BinaryOp::Ne => todo!(),
                frontend::BinaryOp::All => todo!(),
                frontend::BinaryOp::Any => todo!(),
            }
        }
        frontend::Expr::Conditional(_, _, _) => todo!(),
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
