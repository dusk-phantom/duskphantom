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
    ProgramKit {
        program: &mut result,
        env: HashMap::new(),
        fenv: HashMap::new(),
        ctx: HashMap::new(),
    }
    .gen(program)?;
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
    env: HashMap<String, ir::Operand>,
    fenv: HashMap<String, ir::FunPtr>,
    ctx: HashMap<String, ir::ValueType>,
    program: &'a mut Program,
}

/// Kit for translating a function to middle IR.
struct FunctionKit<'a> {
    env: HashMap<String, ir::Operand>,
    fenv: HashMap<String, ir::FunPtr>,
    ctx: HashMap<String, ir::ValueType>,
    program: &'a mut Program,
    entry: ir::BBPtr,
    exit: ir::BBPtr,
}

fn repeat_vec<T>(vec: Vec<T>, n: usize) -> Vec<T>
where
    T: Clone, // The elements of the Vec must implement the Clone trait
{
    let mut result = Vec::new();
    for _ in 0..n {
        result.extend(vec.clone());
    }
    result
}

fn type_to_const(ty: &frontend::Type) -> Result<Vec<ir::Constant>, MiddelError> {
    match ty {
        frontend::Type::Void => todo!(),
        frontend::Type::Int32 => Ok(vec![ir::Constant::Int(0)]),
        frontend::Type::Float32 => Ok(vec![ir::Constant::Float(0.0)]),
        frontend::Type::String => todo!(),
        frontend::Type::Char => todo!(),
        frontend::Type::Boolean => Ok(vec![ir::Constant::Bool(false)]),
        frontend::Type::Pointer(_) => todo!(),
        frontend::Type::Array(ty, num) => Ok(repeat_vec(type_to_const(ty)?, *num)),
        frontend::Type::Function(_, _) => Err(MiddelError::GenError),
        frontend::Type::Enum(_) => todo!(),
        frontend::Type::Union(_) => todo!(),
        frontend::Type::Struct(_) => todo!(),
    }
}

fn expr_to_const(val: &frontend::Expr) -> Result<Vec<ir::Constant>, MiddelError> {
    match val {
        frontend::Expr::Var(_) => todo!(),
        frontend::Expr::Pack(pack) => pack
            .iter()
            // Convert inner expression to constant value
            .map(expr_to_const)
            // Collect as a large result
            .collect::<Result<Vec<Vec<_>>, _>>()
            // Flatten inner vec
            .map(|v| v.into_iter().flatten().collect()),
        frontend::Expr::Map(_) => todo!(),
        frontend::Expr::Index(_, _) => todo!(),
        frontend::Expr::Field(_, _) => todo!(),
        frontend::Expr::Select(_, _) => todo!(),
        frontend::Expr::Int32(i) => Ok(vec![ir::Constant::Int(*i)]),
        frontend::Expr::Float32(f) => Ok(vec![ir::Constant::Float(*f)]),
        frontend::Expr::String(_) => todo!(),
        frontend::Expr::Char(_) => todo!(),
        frontend::Expr::Bool(b) => Ok(vec![ir::Constant::Bool(*b)]),
        frontend::Expr::Call(_, _) => todo!(),
        frontend::Expr::Unary(_, _) => todo!(),
        frontend::Expr::Binary(_, _, _) => todo!(),
        frontend::Expr::Conditional(_, _, _) => todo!(),
    }
}

/// A program kit (top level) can generate declarations
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
            frontend::Decl::Var(ty, id, val) => {
                // Get global variable
                let gval = match val {
                    Some(v) => self.program.mem_pool.new_global_variable(
                        id.clone(),
                        translate_type(ty),
                        // TODO support variable
                        false,
                        expr_to_const(v)?,
                    ),
                    None => self.program.mem_pool.new_global_variable(
                        id.clone(),
                        translate_type(ty),
                        true,
                        type_to_const(ty)?,
                    ),
                };

                // Add global variable to environment
                self.env.insert(id.clone(), ir::Operand::Global(gval));
                Ok(())
            }
            frontend::Decl::Func(ty, id, op) => {
                if let (Some(stmt), frontend::Type::Function(return_ty, params)) = (op, ty) {
                    // Get function type
                    let fty = translate_type(return_ty);

                    // Create function
                    let mut fptr = self.program.mem_pool.new_function(id.clone(), fty);

                    // Fill parameters
                    for param in params.iter() {
                        let param = self.program.mem_pool.new_parameter(
                            param.id.clone().map_or(Err(MiddelError::GenError), Ok)?,
                            translate_type(&param.ty),
                        );
                        fptr.params.push(param);
                    }

                    // Get function block name
                    let fname = self.unique_debug("entry");

                    // Build function
                    let bb = self.program.mem_pool.new_basicblock(fname);
                    let mut kit = FunctionKit {
                        program: self.program,
                        env: self.env.clone(),
                        fenv: self.fenv.clone(),
                        ctx: self.ctx.clone(),
                        entry: bb,
                        exit: bb,
                    };
                    kit.gen_stmt(stmt)?;
                    fptr.entry = Some(kit.entry);
                    fptr.exit = Some(kit.exit);

                    // Add function to environment
                    self.fenv.insert(id.clone(), fptr);
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

    /// Generate a unique debug name for a basic block
    fn unique_debug(&self, base: &'static str) -> String {
        base.to_string()
    }
}

/// A function kit can generate statements
impl<'a> FunctionKit<'a> {
    /// Generate a unique debug name for a basic block
    fn unique_debug(&self, base: &'static str) -> String {
        base.to_string()
    }

    /// Generate a statement into the program
    /// `exit`: previous exit node of a function
    /// Returns: new exit node of a function
    fn gen_stmt(&mut self, stmt: &frontend::Stmt) -> Result<(), MiddelError> {
        match stmt {
            frontend::Stmt::Nothing => Ok(()),
            frontend::Stmt::Decl(decl) => {
                // Generate declaration
                self.gen_decl(decl)
            }
            frontend::Stmt::Expr(op, expr) => {
                // Generate expression
                let operand = self.gen_expr(expr)?;
                match op {
                    // Exist left value, try add result to env
                    Some(lval) => match lval {
                        frontend::LVal::Nothing => todo!(),
                        frontend::LVal::Var(id) => {
                            // Make sure variable is declared
                            let Some(ty) = self.ctx.get(id) else {
                                return Err(MiddelError::GenError);
                            };

                            // Typecheck, TODO type cast
                            if operand.get_type() != *ty {
                                return Err(MiddelError::GenError);
                            }

                            // Add result to env
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
            frontend::Stmt::If(cond, then, alt) => {
                // Evaluate condition
                let operand = self.gen_expr(cond)?;

                // Add br instruction
                let ir::Operand::Instruction(inst) = operand else {
                    todo!("make get_br accept operand")
                };
                let br = self.program.mem_pool.get_br(Some(inst));
                self.exit.push_back(br);

                // Allocate basic blocks
                let then_name = self.unique_debug("then");
                let mut then_bb = self.program.mem_pool.new_basicblock(then_name);
                let alt_name = self.unique_debug("alt");
                let mut alt_bb = self.program.mem_pool.new_basicblock(alt_name);
                let final_name = self.unique_debug("final");
                let final_bb = self.program.mem_pool.new_basicblock(final_name);
                self.exit.set_true_bb(then_bb);
                self.exit.set_false_bb(alt_bb);
                then_bb.set_true_bb(final_bb);
                alt_bb.set_true_bb(final_bb);
                self.exit = final_bb;

                // Generate instructions for branches
                FunctionKit {
                    program: self.program,
                    env: self.env.clone(),
                    fenv: self.fenv.clone(),
                    ctx: self.ctx.clone(),
                    entry: then_bb,
                    exit: then_bb,
                }
                .gen_stmt(then)?;
                FunctionKit {
                    program: self.program,
                    env: self.env.clone(),
                    fenv: self.fenv.clone(),
                    ctx: self.ctx.clone(),
                    entry: alt_bb,
                    exit: alt_bb,
                }
                .gen_stmt(alt)?;
                Ok(())
            }
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
    fn gen_decl(&mut self, decl: &frontend::Decl) -> Result<(), MiddelError> {
        match decl {
            frontend::Decl::Var(ty, id, op) => {
                let mty = translate_type(ty);

                // Add type to context
                self.ctx.insert(id.clone(), mty.clone());
                if let Some(expr) = op {
                    // Generate expression
                    let operand = self.gen_expr(expr)?;

                    // Typecheck, TODO type cast
                    if operand.get_type() != mty {
                        return Err(MiddelError::GenError);
                    }

                    // Add value to environment
                    self.env.insert(id.clone(), operand);
                    Ok(())
                } else {
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
    fn gen_expr(&mut self, expr: &frontend::Expr) -> Result<ir::Operand, MiddelError> {
        match expr {
            frontend::Expr::Var(x) => {
                // Ensure variable is defined
                let Some(operand) = self.env.get(x) else {
                    return Err(MiddelError::GenError);
                };

                // Clone the operand and return, this clones the underlying value or InstPtr
                Ok(operand.clone())
            }
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
                let lop = self.gen_expr(lhs)?;
                let rop = self.gen_expr(rhs)?;
                match op {
                    frontend::BinaryOp::Add => {
                        // Add "add" instruction, operand is the result of the instruction
                        let add = self.program.mem_pool.get_add(lop, rop);
                        Ok(ir::Operand::Instruction(add))
                    }
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
