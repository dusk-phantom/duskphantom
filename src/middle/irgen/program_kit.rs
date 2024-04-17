use crate::errors::MiddelError;
use crate::frontend::{BinaryOp, Decl, Expr, Type, UnaryOp};
use crate::middle::ir::{Constant, FunPtr, ValueType};
use crate::middle::irgen::function_kit::FunctionKit;
use crate::middle::irgen::value::Value;
use crate::middle::irgen::{constant, value_type};
use crate::{frontend, middle};
use std::collections::HashMap;

/// Kit for translating a program to middle IR
pub struct ProgramKit<'a> {
    pub env: HashMap<String, Value>,
    pub fun_env: HashMap<String, FunPtr>,
    pub program: &'a mut middle::Program,
}

/// A program kit (top level) can generate declarations
impl<'a> ProgramKit<'a> {
    pub fn gen(mut self, program: &frontend::Program) -> Result<(), MiddelError> {
        for decl in program.module.iter() {
            self.gen_decl(decl)?;
        }
        for decl in program.module.iter() {
            self.gen_impl(decl)?;
        }
        Ok(())
    }

    /// Generate a declaration into the program
    /// Fails when declaration does not have a name
    pub fn gen_decl(&mut self, decl: &Decl) -> Result<(), MiddelError> {
        match decl {
            Decl::Var(ty, id, val) | Decl::Const(ty, id, val) => {
                // Get if value is global variable or constant
                let is_global_variable: bool = match decl {
                    Decl::Var(_, _, _) => true,
                    Decl::Const(_, _, _) => false,
                    _ => false,
                };

                // Get initializer
                let initializer = match val {
                    Some(v) => constant::expr_to_const(v)?,
                    None => constant::type_to_const(ty)?,
                };

                // Get global variable
                let global_val = self.program.mem_pool.new_global_variable(
                    id.clone(),
                    value_type::translate_type(ty),
                    is_global_variable,
                    initializer,
                );

                // Add global variable (pointer) to environment
                self.env
                    .insert(id.clone(), Value::Pointer(global_val.into()));

                // Add global variable to program
                self.program.module.global_variables.push(global_val);
                Ok(())
            }
            Decl::Func(Type::Function(return_ty, _), id, _) => {
                // Get function type
                let fty = value_type::translate_type(return_ty);

                // Create function
                let fun_ptr = self.program.mem_pool.new_function(id.clone(), fty.clone());

                // Add function to environment
                self.fun_env.insert(id.clone(), fun_ptr);

                // Add function to program
                self.program.module.functions.push(fun_ptr);
                Ok(())
            }
            _ => Err(MiddelError::GenError),
        }
    }

    /// Generate an implementation into the program
    pub fn gen_impl(&mut self, decl: &Decl) -> Result<(), MiddelError> {
        match decl {
            Decl::Func(Type::Function(_, params), id, Some(stmt)) => {
                // Clone function env before mutating it
                let cloned_fun_env = self.fun_env.clone();

                // Get function and its type
                let fun_ptr = self.fun_env.get_mut(id).ok_or(MiddelError::GenError)?;
                let fty = fun_ptr.return_type.clone();

                // Create basic block
                let entry_name = "entry".to_string();
                let mut entry = self.program.mem_pool.new_basicblock(entry_name);

                // Fill parameters
                for param in params.iter() {
                    let param = self.program.mem_pool.new_parameter(
                        param.id.clone().ok_or(MiddelError::GenError)?,
                        value_type::translate_type(&param.ty),
                    );

                    // Add parameter to function
                    fun_ptr.params.push(param);

                    // Add parameter to entry
                    let alloc = self
                        .program
                        .mem_pool
                        .get_alloca(param.value_type.clone(), 1);
                    let store = self.program.mem_pool.get_store(param.into(), alloc.into());
                    entry.push_back(alloc);
                    entry.push_back(store);

                    // Add parameter to env
                    self.env
                        .insert(param.name.clone(), Value::Pointer(alloc.into()));
                }

                // Build function
                let mut counter: usize = 0;
                let mut kit = FunctionKit {
                    program: self.program,
                    env: self.env.clone(),
                    fun_env: cloned_fun_env,
                    entry,
                    exit: entry,
                    break_to: None,
                    continue_to: None,
                    return_type: fty,
                    counter: &mut counter,
                };
                kit.gen_stmt(stmt)?;
                fun_ptr.entry = Some(kit.entry);
                fun_ptr.exit = Some(kit.exit);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Generate constant expression
    pub fn gen_expr(&mut self, expr: &Expr) -> Result<Value, MiddelError> {
        match expr {
            Expr::Var(x) => {
                // Ensure variable is defined
                let Some(operand) = self.env.get(x) else {
                    return Err(MiddelError::GenError);
                };

                // Clone the operand and return, this clones the underlying value or InstPtr
                Ok(operand.clone())
            }
            // Some memory copy operation is required to process arrays
            Expr::Pack(_) => Err(MiddelError::GenError),
            Expr::Map(_) => Err(MiddelError::GenError),
            Expr::Index(x, v) => Err(MiddelError::GenError),
            Expr::Field(_, _) => Err(MiddelError::GenError),
            Expr::Select(_, _) => Err(MiddelError::GenError),
            Expr::Int32(x) => Ok(Constant::Int(*x).into()),
            Expr::Float32(x) => Ok(Constant::Float(*x).into()),
            Expr::String(_) => Err(MiddelError::GenError),
            Expr::Char(_) => Err(MiddelError::GenError),
            Expr::Bool(_) => Err(MiddelError::GenError),
            Expr::Call(func, args) => Err(MiddelError::GenError),
            Expr::Unary(op, expr) => self.gen_unary(op, expr),
            Expr::Binary(op, lhs, rhs) => self.gen_binary(op, lhs, rhs),
            Expr::Conditional(_, _, _) => Err(MiddelError::GenError),
        }
    }

    /// Generate a unary expression
    pub fn gen_unary(&mut self, op: &UnaryOp, expr: &Expr) -> Result<Value, MiddelError> {
        // Generate constant
        let val = self.gen_expr(expr)?;

        // Apply operation
        match op {
            UnaryOp::Neg => match val {
                Constant::Int(i) => Ok(Constant::Int(-i).into()),
                Constant::Float(i) => Ok(Constant::Float(-i).into()),
                Constant::Bool(i) => Ok(Constant::Int(-Into::<i32>::into(i)).into()),
            },
            UnaryOp::Pos => Ok(val.into()),
            UnaryOp::Not => match val {
                Constant::Int(i) => Ok(Constant::Bool(i != 0).into()),
                Constant::Float(i) => Ok(Constant::Bool(i != 0.0).into()),
                Constant::Bool(i) => Ok(Constant::Bool(i).into()),
            },
            _ => Err(MiddelError::GenError),
        }
    }

    /// Generate a binary expression
    pub fn gen_binary(
        &mut self,
        op: &BinaryOp,
        lhs: &Expr,
        rhs: &Expr,
    ) -> Result<Value, MiddelError> {
        // Generate constants
        let lhs_val = self.gen_expr(lhs)?.constant()?;
        let rhs_val = self.gen_expr(rhs)?.constant()?;

        // Calculate maximum type for operator polymorphism
        let max_ty = lhs_val.get_type().clone().max_with(&rhs_val.get_type());

        // Apply operation
        match op {
            BinaryOp::Add => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add "add" instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self.program.mem_pool.get_add(lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self.program.mem_pool.get_fadd(lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    _ => Err(MiddelError::GenError),
                }
            }
            BinaryOp::Sub => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add "sub" instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self.program.mem_pool.get_sub(lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self.program.mem_pool.get_fsub(lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    _ => Err(MiddelError::GenError),
                }
            }
            BinaryOp::Mul => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add "mul" instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self.program.mem_pool.get_mul(lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self.program.mem_pool.get_fmul(lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    _ => Err(MiddelError::GenError),
                }
            }
            BinaryOp::Div => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add "div" instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self.program.mem_pool.get_sdiv(lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self.program.mem_pool.get_fdiv(lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    _ => Err(MiddelError::GenError),
                }
            }
            BinaryOp::Mod => {
                // Load operand as integers
                let lop = lhs_val.load(ValueType::Int, self)?;
                let rop = rhs_val.load(ValueType::Int, self)?;

                // Add "signed rem" instruction, operand is the result of the instruction
                let inst = self.program.mem_pool.get_srem(lop, rop);
                self.exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
            // Bitwise operation on int is not required
            BinaryOp::Shr => Err(MiddelError::GenError),
            BinaryOp::Shl => Err(MiddelError::GenError),
            BinaryOp::BitAnd => Err(MiddelError::GenError),
            BinaryOp::BitOr => Err(MiddelError::GenError),
            BinaryOp::BitXor => Err(MiddelError::GenError),
            BinaryOp::Gt => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add compare instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_icmp(ICmpOp::Sgt, max_ty, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fcmp(FCmpOp::Ugt, max_ty, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    _ => Err(MiddelError::GenError),
                }
            }
            BinaryOp::Lt => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add compare instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_icmp(ICmpOp::Slt, max_ty, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fcmp(FCmpOp::Ult, max_ty, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    _ => Err(MiddelError::GenError),
                }
            }
            BinaryOp::Ge => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add compare instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_icmp(ICmpOp::Sge, max_ty, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fcmp(FCmpOp::Uge, max_ty, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    _ => Err(MiddelError::GenError),
                }
            }
            BinaryOp::Le => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add compare instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_icmp(ICmpOp::Sle, max_ty, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fcmp(FCmpOp::Ule, max_ty, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    _ => Err(MiddelError::GenError),
                }
            }
            BinaryOp::Eq => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add compare instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self.program.mem_pool.get_icmp(ICmpOp::Eq, max_ty, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fcmp(FCmpOp::Ueq, max_ty, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    _ => Err(MiddelError::GenError),
                }
            }
            BinaryOp::Ne => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add compare instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self.program.mem_pool.get_icmp(ICmpOp::Ne, max_ty, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fcmp(FCmpOp::Une, max_ty, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    _ => Err(MiddelError::GenError),
                }
            }
            BinaryOp::And => {
                // Load operands as bool
                let lop = lhs_val.load(ValueType::Bool, self)?;
                let rop = rhs_val.load(ValueType::Bool, self)?;

                // Add "and" instruction, operand is the result of the instruction
                let inst = self.program.mem_pool.get_and(lop, rop);
                self.exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
            BinaryOp::Or => {
                // Load operands as bool
                let lop = lhs_val.load(ValueType::Bool, self)?;
                let rop = rhs_val.load(ValueType::Bool, self)?;

                // Add "or" instruction, operand is the result of the instruction
                let inst = self.program.mem_pool.get_or(lop, rop);
                self.exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
        }
    }
}

