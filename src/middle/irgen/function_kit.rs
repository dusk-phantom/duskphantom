use crate::errors::MiddleError;
use crate::frontend::{BinaryOp, Decl, Expr, Stmt, UnaryOp};
use crate::middle;
use crate::middle::ir::instruction::misc_inst::{FCmpOp, ICmpOp};
use crate::middle::ir::{BBPtr, Constant, FunPtr, ValueType};
use crate::middle::irgen::value::Value;
use crate::middle::irgen::{value, value_type};
use std::collections::HashMap;

/// Kit for translating a function to middle IR
pub struct FunctionKit<'a> {
    pub env: HashMap<String, Value>,
    pub fun_env: HashMap<String, FunPtr>,
    pub program: &'a mut middle::Program,
    pub exit: BBPtr,
    pub break_to: Option<BBPtr>,
    pub continue_to: Option<BBPtr>,
    pub return_type: ValueType,
    pub counter: &'a mut usize,
}

/// A function kit can generate statements
impl<'a> FunctionKit<'a> {
    /// Generate a new function kit
    pub fn gen_function_kit(
        &mut self,
        exit: BBPtr,
        break_to: Option<BBPtr>,
        continue_to: Option<BBPtr>,
    ) -> FunctionKit {
        FunctionKit {
            program: self.program,
            env: self.env.clone(),
            fun_env: self.fun_env.clone(),
            exit,
            break_to,
            continue_to,
            return_type: self.return_type.clone(),
            counter: self.counter,
        }
    }

    /// Generate a unique basic block name
    pub fn unique_name(&mut self, base: &str) -> String {
        let name = format!("{}{}", base, self.counter);
        *self.counter += 1;
        name
    }

    /// Generate a statement into the program
    ///
    /// Returns the exit of the statement block that can be appended to
    /// eg. if "break", the exit block can't be appended, this will return `None`
    /// The exit returned is different from kit.exit, which always exists
    ///
    /// Error when statement generation is not successful
    pub fn gen_stmt(&mut self, stmt: &Stmt) -> Result<Option<BBPtr>, MiddleError> {
        match stmt {
            Stmt::Nothing => (),
            Stmt::Decl(decl) => {
                // Generate declaration
                self.gen_decl(decl)?;
            }
            Stmt::Expr(opt_lhs, expr) => {
                // Generate expression
                let rhs = self.gen_expr(expr)?;

                // Try to assign if exists left value
                if let Some(lhs) = opt_lhs {
                    self.gen_expr(lhs)?.assign(self, rhs)?;
                }
            }
            Stmt::If(cond, then, alt) => {
                // Allocate basic blocks
                let cond_name = self.unique_name("cond");
                let mut cond_bb = self.program.mem_pool.new_basicblock(cond_name);
                let then_name = self.unique_name("then");
                let then_entry = self.program.mem_pool.new_basicblock(then_name);
                let alt_name = self.unique_name("alt");
                let alt_entry = self.program.mem_pool.new_basicblock(alt_name);
                let final_name = self.unique_name("final");
                let final_bb = self.program.mem_pool.new_basicblock(final_name);

                // Redirect exit to condition block
                self.exit.set_true_bb(cond_bb);
                self.exit.push_back(self.program.mem_pool.get_br(None));

                // Add condition and br to condition block
                self.exit = cond_bb;
                let operand = self.gen_expr(cond)?.load(ValueType::Bool, self)?;
                let br = self.program.mem_pool.get_br(Some(operand));
                cond_bb.push_back(br);
                cond_bb.set_true_bb(then_entry);
                cond_bb.set_false_bb(alt_entry);

                // Add statements and br to then branch
                // Retain break_to and continue_to
                let then_exit = self
                    .gen_function_kit(then_entry, self.break_to, self.continue_to)
                    .gen_stmt(then)?;
                if let Some(mut then_exit) = then_exit {
                    then_exit.push_back(self.program.mem_pool.get_br(None));
                    then_exit.set_true_bb(final_bb);
                }

                // Add statements and br to alt branch
                let alt_exit = self
                    .gen_function_kit(alt_entry, self.break_to, self.continue_to)
                    .gen_stmt(alt)?;
                if let Some(mut alt_exit) = alt_exit {
                    alt_exit.push_back(self.program.mem_pool.get_br(None));
                    alt_exit.set_true_bb(final_bb);
                }

                // Exit is final block
                self.exit = final_bb;
            }
            Stmt::While(cond, body) => {
                // Allocate basic blocks
                let cond_name = self.unique_name("cond");
                let mut cond_bb = self.program.mem_pool.new_basicblock(cond_name);
                let body_name = self.unique_name("body");
                let body_entry = self.program.mem_pool.new_basicblock(body_name);
                let final_name = self.unique_name("final");
                let final_bb = self.program.mem_pool.new_basicblock(final_name);

                // Redirect current exit to condition block
                self.exit.set_true_bb(cond_bb);
                self.exit.push_back(self.program.mem_pool.get_br(None));

                // Add statements and br to body block
                let body_exit = self
                    .gen_function_kit(body_entry, Some(final_bb), Some(cond_bb))
                    .gen_stmt(body)?;
                if let Some(mut body_exit) = body_exit {
                    body_exit.push_back(self.program.mem_pool.get_br(None));
                    body_exit.set_true_bb(cond_bb);
                }

                // Add condition and br to condition block
                self.exit = cond_bb;
                cond_bb.set_true_bb(body_entry);
                cond_bb.set_false_bb(final_bb);
                let operand = self.gen_expr(cond)?.load(ValueType::Bool, self)?;
                let br = self.program.mem_pool.get_br(Some(operand));
                cond_bb.push_back(br);

                // Exit is final block
                self.exit = final_bb;
            }
            Stmt::DoWhile(body, cond) => {
                // Allocate basic blocks
                let body_name = self.unique_name("body");
                let body_entry = self.program.mem_pool.new_basicblock(body_name);
                let cond_name = self.unique_name("cond");
                let mut cond_bb = self.program.mem_pool.new_basicblock(cond_name);
                let final_name = self.unique_name("final");
                let final_bb = self.program.mem_pool.new_basicblock(final_name);

                // Redirect current exit to body block
                self.exit.set_true_bb(body_entry);
                self.exit.push_back(self.program.mem_pool.get_br(None));

                // Add statements and br to body block
                let body_exit = self
                    .gen_function_kit(body_entry, Some(final_bb), Some(cond_bb))
                    .gen_stmt(body)?;
                if let Some(mut body_exit) = body_exit {
                    body_exit.push_back(self.program.mem_pool.get_br(None));
                    body_exit.set_true_bb(cond_bb);
                }

                // Add condition and br to condition block
                self.exit = cond_bb;
                cond_bb.set_true_bb(body_entry);
                cond_bb.set_false_bb(final_bb);
                let operand = self.gen_expr(cond)?.load(ValueType::Bool, self)?;
                let br = self.program.mem_pool.get_br(Some(operand));
                cond_bb.push_back(br);

                // Exit is final block
                self.exit = final_bb;
            }
            Stmt::For(_, _, _, _) => return Err(MiddleError::GenError),
            Stmt::Break => {
                // Add br instruction to exit block
                let br = self.program.mem_pool.get_br(None);
                self.exit.push_back(br);

                // When break statement appears, break_to must not be None
                let Some(break_to) = self.break_to else {
                    return Err(MiddleError::GenError);
                };

                // Rewrite next block to break destination
                self.exit.set_true_bb(break_to);

                // Return None to indicate that the exit block can't be appended
                return Ok(None);
            }
            Stmt::Continue => {
                // Add br instruction to exit block
                let br = self.program.mem_pool.get_br(None);
                self.exit.push_back(br);

                // When continue statement appears, continue_to must not be None
                let Some(continue_to) = self.continue_to else {
                    return Err(MiddleError::GenError);
                };

                // Rewrite next block to continue destination
                self.exit.set_true_bb(continue_to);

                // Return None to indicate that the exit block can't be appended
                return Ok(None);
            }
            Stmt::Return(expr) => {
                // Add returned result to exit block
                let return_value = match expr {
                    Some(expr) => Some(self.gen_expr(expr)?.load(self.return_type.clone(), self)?),
                    None => None,
                };

                // Add ret instruction to exit block
                let ret = self.program.mem_pool.get_ret(return_value);
                self.exit.push_back(ret);

                // Return None to indicate that the exit block can't be appended
                return Ok(None);
            }
            Stmt::Block(stmts) => {
                // Add statements to current block
                for stmt in stmts.iter() {
                    if self.gen_stmt(stmt)?.is_none() {
                        // The rest of the code in this block will not be executed
                        // because the exit block can't be appended
                        return Ok(None);
                    }
                }
            }
        }
        Ok(Some(self.exit))
    }

    /// Generate a declaration as a statement into the program
    pub fn gen_decl(&mut self, decl: &Decl) -> Result<(), MiddleError> {
        match decl {
            Decl::Var(raw_ty, id, op) => {
                // Allocate space for variable, add to environment
                let ty = value_type::translate_type(raw_ty);
                let lhs = value::alloc(ty.clone(), self);
                self.env.insert(id.clone(), lhs.clone());

                // Assign to the variable if it is defined
                if let Some(expr) = op {
                    // Generate expression as variable type
                    let rhs = self.gen_expr(expr)?;

                    // Assign operand to value
                    lhs.assign(self, rhs)?;
                };
                Ok(())
            }
            Decl::Stack(decls) => {
                // Generate each declaration
                for decl in decls.iter() {
                    self.gen_decl(decl)?;
                }
                Ok(())
            }
            _ => Err(MiddleError::GenError),
        }
    }

    /// Generate an expression as a statement into the program
    pub fn gen_expr(&mut self, expr: &Expr) -> Result<Value, MiddleError> {
        match expr {
            Expr::Var(x) => {
                // Ensure variable is defined
                let Some(operand) = self.env.get(x) else {
                    return Err(MiddleError::GenError);
                };

                // Clone the operand and return, this clones the underlying value or InstPtr
                Ok(operand.clone())
            }
            Expr::Pack(ls) => Ok(Value::Array(
                ls.iter()
                    .map(|x| self.gen_expr(x))
                    .collect::<Result<_, _>>()?,
            )),
            Expr::Map(_) => Err(MiddleError::GenError),
            Expr::Index(x, v) => {
                // Load index as integer
                let ix = self.gen_expr(v)?.load(ValueType::Int, self)?;

                // Generate GEP
                self.gen_expr(x)?
                    .get_element_ptr(self, vec![Constant::Int(0).into(), ix])
            }
            Expr::Field(_, _) => Err(MiddleError::GenError),
            Expr::Select(_, _) => Err(MiddleError::GenError),
            Expr::Int32(x) => Ok(Constant::Int(*x).into()),
            Expr::Float32(x) => Ok(Constant::Float(*x).into()),
            Expr::String(_) => Err(MiddleError::GenError),
            Expr::Char(_) => Err(MiddleError::GenError),
            Expr::Bool(_) => Err(MiddleError::GenError),
            Expr::Call(func, args) => {
                // Generate arguments
                let mut operands = Vec::new();
                for arg in args.iter() {
                    operands.push(self.gen_expr(arg)?.load(ValueType::Int, self)?);
                }

                // Ensure function is a defined variable
                let Expr::Var(func) = *func.clone() else {
                    return Err(MiddleError::GenError);
                };
                let Some(fun) = self.fun_env.get(&func) else {
                    return Err(MiddleError::GenError);
                };

                // Call the function
                let inst = self.program.mem_pool.get_call(*fun, operands);
                self.exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
            Expr::Unary(op, expr) => self.gen_unary(op, expr),
            Expr::Binary(op, lhs, rhs) => self.gen_binary(op, lhs, rhs),
            Expr::Conditional(_, _, _) => Err(MiddleError::GenError),
        }
    }

    /// Generate a unary expression
    pub fn gen_unary(&mut self, op: &UnaryOp, expr: &Expr) -> Result<Value, MiddleError> {
        // Generate argument
        let val = self.gen_expr(expr)?;

        // Calculate type for operator polymorphism
        let ty = val.get_type();

        // Apply operation
        match op {
            UnaryOp::Neg => {
                // Return 0 - x
                let operand = val.load(ty.clone(), self)?;
                match ty {
                    ValueType::Int => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_sub(Constant::Int(0).into(), operand);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fsub(Constant::Float(0.0).into(), operand);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    ValueType::Bool => {
                        // Convert to int and then make negative
                        let zext = self.program.mem_pool.get_zext(operand);
                        let sub = self
                            .program
                            .mem_pool
                            .get_sub(Constant::Int(0).into(), zext.into());
                        self.exit.push_back(zext);
                        self.exit.push_back(sub);
                        Ok(Value::Operand(sub.into()))
                    }
                    _ => Err(MiddleError::GenError),
                }
            }
            UnaryOp::Pos => {
                // Return operand directly
                let operand = val.load(ty.clone(), self)?;
                match ty {
                    ValueType::Int | ValueType::Float | ValueType::Bool => {
                        Ok(Value::Operand(operand))
                    }
                    _ => Err(MiddleError::GenError),
                }
            }
            UnaryOp::Not => {
                // Load as boolean
                let bool_op = val.load(ValueType::Bool, self)?;

                // Add "xor" instruction
                let inst = self
                    .program
                    .mem_pool
                    .get_xor(bool_op, Constant::Bool(true).into());
                self.exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
            _ => Err(MiddleError::GenError),
        }
    }

    /// Generate a binary expression
    pub fn gen_binary(
        &mut self,
        op: &BinaryOp,
        lhs: &Expr,
        rhs: &Expr,
    ) -> Result<Value, MiddleError> {
        // Generate arguments
        let lhs_val = self.gen_expr(lhs)?;
        let rhs_val = self.gen_expr(rhs)?;

        // Calculate maximum type for operator polymorphism
        let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

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
                    _ => Err(MiddleError::GenError),
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
                    _ => Err(MiddleError::GenError),
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
                    _ => Err(MiddleError::GenError),
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
                    _ => Err(MiddleError::GenError),
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
            BinaryOp::Shr => Err(MiddleError::GenError),
            BinaryOp::Shl => Err(MiddleError::GenError),
            BinaryOp::BitAnd => Err(MiddleError::GenError),
            BinaryOp::BitOr => Err(MiddleError::GenError),
            BinaryOp::BitXor => Err(MiddleError::GenError),
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
                    _ => Err(MiddleError::GenError),
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
                    _ => Err(MiddleError::GenError),
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
                    _ => Err(MiddleError::GenError),
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
                    _ => Err(MiddleError::GenError),
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
                    _ => Err(MiddleError::GenError),
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
                    _ => Err(MiddleError::GenError),
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
