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
    let _ = ProgramKit {
        program: &mut result,
        env: HashMap::new(),
    }
    .gen(program);
    Ok(result)
}

/// Translate a frontend type to IR value type
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

/// Kit for translating a program to middle IR.
struct ProgramKit<'a> {
    program: &'a mut Program,
    env: HashMap<String, ir::Operand>,
}

// Kit for translating a function to middle IR.
struct FunctionKit<'a> {
    program: &'a mut Program,
    env: HashMap<String, ir::Operand>,
    entry: ir::BBPtr,
    exit: ir::BBPtr,
}

impl<'a> ProgramKit<'a> {
    pub fn gen(mut self, program: &frontend::Program) -> Result<(), MiddelError> {
        for decl in program.module.iter() {
            self.gen_decl(decl)?;
        }
        Ok(())
    }

    /// Generate a declaration into the program
    /// Fails when declaration does not have a name
    fn gen_decl(&mut self, decl: &frontend::Decl) -> Result<(), MiddelError> {
        match decl {
            frontend::Decl::Var(_, _, _) => todo!(),
            frontend::Decl::Func(ty, id, op) => {
                if let (Some(stmt), frontend::Type::Function(return_ty, params)) = (op, ty) {
                    // Get function type
                    let fty = translate_type(return_ty);

                    // Create function
                    let mut fptr = self.program.mem_pool.new_function(id.clone(), fty);

                    // Fill parameters
                    for param in params.iter() {
                        fptr.params.push(ir::Parameter {
                            name: param.id.clone().map_or(Err(MiddelError::GenError), Ok)?,
                            value_type: translate_type(&param.ty),
                        })
                    }

                    // Get function block name
                    let fname = self.unique_debug("entry");

                    // Build function
                    let bb = self.program.mem_pool.new_basicblock(fname);
                    let mut kit = FunctionKit {
                        program: self.program,
                        env: self.env.clone(),
                        entry: bb,
                        exit: bb,
                    };
                    kit.gen_stmt(stmt)?;
                    fptr.entry = Some(kit.entry);
                    fptr.exit = Some(kit.exit);
                    Ok(())
                } else {
                    Ok(())
                }
            }
            frontend::Decl::Enum(_, _) => todo!(),
            frontend::Decl::Union(_, _) => todo!(),
            frontend::Decl::Struct(_, _) => todo!(),
        }
    }

    fn unique_debug(&self, base: &'static str) -> String {
        base.to_string()
    }

    fn unique_name(&self) -> String {
        "".to_string()
    }
}

impl<'a> FunctionKit<'a> {
    /// Generate a statement into the program
    /// `exit`: previous exit node of a function
    /// Returns: new exit node of a function
    fn gen_stmt(&mut self, stmt: &frontend::Stmt) -> Result<(), MiddelError> {
        match stmt {
            frontend::Stmt::Nothing => Ok(()),
            frontend::Stmt::Decl(decl) => {
                // Insert created declaration to environment
                self.gen_stml_decl(decl)
            }
            frontend::Stmt::Expr(op, expr) => {
                // Evaluate expression
                let operand = self.gen_stmt_expr(expr)?;
                match op {
                    // Exist left value, add result to env
                    Some(lval) => match lval {
                        frontend::LVal::Nothing => todo!(),
                        frontend::LVal::Var(id) => {
                            self.env.insert(id.clone(), operand);
                            Ok(())
                        }
                        frontend::LVal::Index(_, _) => todo!(),
                        frontend::LVal::Call(_, _) => todo!(),
                        frontend::LVal::Pointer(_) => todo!(),
                    },
                    // No left value, discard result
                    None => Ok(()),
                }
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

    /// Generate a declaration as a statement into the program
    /// `exit`: previous exit node of the function
    /// Returns: declaration name, declared variable, new exit node of the function
    fn gen_stml_decl(&mut self, decl: &frontend::Decl) -> Result<(), MiddelError> {
        match decl {
            frontend::Decl::Var(ty, id, op) => {
                let mty = translate_type(ty);
                if let Some(expr) = op {
                    // Directly generate expression
                    let operand = self.gen_stmt_expr(expr)?;
                    // Add to environment
                    self.env.insert(id.clone(), operand);
                    Ok(())
                } else {
                    // Alloc variable
                    let alloca = self.program.mem_pool.get_alloca(mty.clone(), 1);
                    self.exit.push_back(alloca);
                    // Ok((id.clone(), ir::Operand::Instruction(alloca), *exit))
                    Ok(())
                }
            }
            frontend::Decl::Func(_, _, _) => todo!(),
            frontend::Decl::Enum(_, _) => todo!(),
            frontend::Decl::Union(_, _) => todo!(),
            frontend::Decl::Struct(_, _) => todo!(),
        }
    }

    /// Generate a expression as a statement into the program
    /// `exit`: previous exit node of the function
    /// Returns: calculated variable, new exit node of the function
    fn gen_stmt_expr(&mut self, expr: &frontend::Expr) -> Result<ir::Operand, MiddelError> {
        match expr {
            frontend::Expr::Var(x) => Ok(self.env[x].clone()),
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
                let lop = self.gen_stmt_expr(lhs)?;
                let rop = self.gen_stmt_expr(rhs)?;
                match op {
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
