use crate::errors::MiddelError;
use crate::frontend;
use crate::frontend::{BinaryOp, Decl, Expr, Stmt};
use crate::middle::ir::{Constant, Operand, ValueType};
use crate::middle::ir::instruction::misc_inst::ICmpOp;
use crate::middle::irgen;
use crate::middle::irgen::{FunctionKit, Value};

/// A function kit can generate statements
impl<'a> FunctionKit<'a> {
    /// Generate a unique debug name for a basic block
    pub fn unique_debug(&self, base: &'static str) -> String {
        base.to_string()
    }

    /// Generate a statement into the program
    /// `exit`: previous exit node of a function
    /// Returns: new exit node of a function
    pub fn gen_stmt(&mut self, stmt: &Stmt) -> Result<(), MiddelError> {
        match stmt {
            Stmt::Nothing => Ok(()),
            Stmt::Decl(decl) => {
                // Generate declaration
                self.gen_decl(decl)
            }
            Stmt::Expr(opt_lval, expr) => {
                // Generate expression
                let operand = self.gen_expr(expr)?.load(self);
                match opt_lval {
                    // Exist left value, try add result to env
                    Some(lval) => match lval {
                        frontend::LVal::Nothing => todo!(),
                        frontend::LVal::Var(id) => {
                            // Find variable in environment
                            let val = {
                                let Some(v @ Value::Pointer(_)) = self.env.get(id) else {
                                    return Err(MiddelError::GenError);
                                };
                                v.clone()
                            };

                            // Type check and type cast
                            let operand = operand.conv(val.get_type(), self)?;

                            // Assign to value
                            val.assign(self, operand)?;
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
            Stmt::If(cond, then, alt) => {
                // Allocate basic blocks
                let cond_name = self.unique_debug("cond");
                let mut cond_bb = self.program.mem_pool.new_basicblock(cond_name);
                let then_name = self.unique_debug("then");
                let mut then_bb = self.program.mem_pool.new_basicblock(then_name);
                let alt_name = self.unique_debug("alt");
                let mut alt_bb = self.program.mem_pool.new_basicblock(alt_name);
                let final_name = self.unique_debug("final");
                let final_bb = self.program.mem_pool.new_basicblock(final_name);

                // Route basic blocks
                cond_bb.set_true_bb(then_bb);
                cond_bb.set_false_bb(alt_bb);
                then_bb.set_true_bb(final_bb);
                alt_bb.set_true_bb(final_bb);
                self.exit.set_true_bb(cond_bb);

                // Add br to exit block, jump to condition block
                self.exit.push_back(self.program.mem_pool.get_br(None));
                self.exit = cond_bb;

                // Add condition and br to condition block
                let operand = self.gen_expr(cond)?.load(self);
                let br = self.program.mem_pool.get_br(Some(operand));
                cond_bb.push_back(br);

                // Add statements and br to then branch
                FunctionKit {
                    program: self.program,
                    env: self.env.clone(),
                    fun_env: self.fun_env.clone(),
                    ctx: self.ctx.clone(),
                    entry: then_bb,
                    exit: then_bb,
                    break_to: None,
                    continue_to: None,
                    return_type: self.return_type.clone(),
                }
                .gen_stmt(then)?;
                then_bb.push_back(self.program.mem_pool.get_br(None));

                // Add statements and br to alt branch
                FunctionKit {
                    program: self.program,
                    env: self.env.clone(),
                    fun_env: self.fun_env.clone(),
                    ctx: self.ctx.clone(),
                    entry: alt_bb,
                    exit: alt_bb,
                    break_to: None,
                    continue_to: None,
                    return_type: self.return_type.clone(),
                }
                .gen_stmt(alt)?;
                alt_bb.push_back(self.program.mem_pool.get_br(None));

                // Increment exit
                self.exit = final_bb;
                Ok(())
            }
            Stmt::While(cond, body) => {
                // Allocate basic blocks
                let cond_name = self.unique_debug("cond");
                let mut cond_bb = self.program.mem_pool.new_basicblock(cond_name);
                let body_name = self.unique_debug("body");
                let mut body_bb = self.program.mem_pool.new_basicblock(body_name);
                let final_name = self.unique_debug("final");
                let final_bb = self.program.mem_pool.new_basicblock(final_name);

                // Route basic blocks
                cond_bb.set_true_bb(body_bb);
                cond_bb.set_false_bb(final_bb);
                body_bb.set_true_bb(cond_bb);
                self.exit.set_true_bb(cond_bb);

                // Add br to exit block, jump to condition block
                self.exit.push_back(self.program.mem_pool.get_br(None));
                self.exit = cond_bb;

                // Add condition and br to condition block
                let operand = self.gen_expr(cond)?.load(self);
                let br = self.program.mem_pool.get_br(Some(operand));
                cond_bb.push_back(br);

                // Add statements and br to body block
                FunctionKit {
                    program: self.program,
                    env: self.env.clone(),
                    fun_env: self.fun_env.clone(),
                    ctx: self.ctx.clone(),
                    entry: body_bb,
                    exit: body_bb,
                    break_to: Some(final_bb),
                    continue_to: Some(cond_bb),
                    return_type: self.return_type.clone(),
                }
                .gen_stmt(body)?;
                body_bb.push_back(self.program.mem_pool.get_br(None));

                // Increment exit
                self.exit = final_bb;
                Ok(())
            }
            Stmt::DoWhile(body, cond) => {
                // Allocate basic blocks
                let body_name = self.unique_debug("body");
                let mut body_bb = self.program.mem_pool.new_basicblock(body_name);
                let cond_name = self.unique_debug("cond");
                let mut cond_bb = self.program.mem_pool.new_basicblock(cond_name);
                let final_name = self.unique_debug("final");
                let final_bb = self.program.mem_pool.new_basicblock(final_name);

                // Route basic blocks
                body_bb.set_true_bb(cond_bb);
                cond_bb.set_true_bb(body_bb);
                cond_bb.set_false_bb(final_bb);
                self.exit.set_true_bb(body_bb);

                // Add br to exit block, jump to condition block
                self.exit.push_back(self.program.mem_pool.get_br(None));
                self.exit = cond_bb;

                // Add condition and br to condition block
                let operand = self.gen_expr(cond)?.load(self);
                let br = self.program.mem_pool.get_br(Some(operand));
                cond_bb.push_back(br);

                // Add statements and br to body block
                FunctionKit {
                    program: self.program,
                    env: self.env.clone(),
                    fun_env: self.fun_env.clone(),
                    ctx: self.ctx.clone(),
                    entry: body_bb,
                    exit: body_bb,
                    break_to: Some(final_bb),
                    continue_to: Some(cond_bb),
                    return_type: self.return_type.clone(),
                }
                .gen_stmt(body)?;
                body_bb.push_back(self.program.mem_pool.get_br(None));

                // Increment exit
                self.exit = final_bb;
                Ok(())
            }
            Stmt::For(_, _, _, _) => todo!(),
            Stmt::Break => {
                // Add br instruction to exit block
                let br = self.program.mem_pool.get_br(None);
                self.exit.push_back(br);

                // When break statement appears, break_to must not be None
                let Some(break_to) = self.break_to else {
                    return Err(MiddelError::GenError);
                };

                // Rewrite next block to break destination
                self.exit.set_true_bb(break_to);
                Ok(())
            }
            Stmt::Continue => {
                // Add br instruction to exit block
                let br = self.program.mem_pool.get_br(None);
                self.exit.push_back(br);

                // When continue statement appears, continue_to must not be None
                let Some(continue_to) = self.continue_to else {
                    return Err(MiddelError::GenError);
                };

                // Rewrite next block to continue destination
                self.exit.set_true_bb(continue_to);
                Ok(())
            }
            Stmt::Return(expr) => {
                // Add returned result to exit block
                let return_value = match expr {
                    Some(expr) => {
                        Some(self.gen_expr(expr)?.load(self).conv(self.return_type.clone(), self)?)
                    }
                    None => None,
                };

                // Add ret instruction to exit block
                let ret = self.program.mem_pool.get_ret(return_value);
                self.exit.push_back(ret);
                Ok(())
            }
            Stmt::Block(stmts) => {
                // Add statements to current block
                for stmt in stmts.iter() {
                    self.gen_stmt(stmt)?;
                }
                Ok(())
            }
        }
    }

    /// Generate a declaration as a statement into the program
    /// `exit`: previous exit node of the function
    /// Returns: declaration name, declared variable, new exit node of the function
    pub fn gen_decl(&mut self, decl: &Decl) -> Result<(), MiddelError> {
        match decl {
            Decl::Var(raw_ty, id, op) => {
                // Add type to context
                let ty = irgen::translate_type(raw_ty);
                self.ctx.insert(id.clone(), ty.clone());

                // Allocate space for variable, add to environment
                let val = irgen::alloc(ty.clone(), self);
                self.env.insert(id.clone(), val.clone());

                // Assign to the variable if it is defined
                if let Some(expr) = op {
                    // Generate expression
                    let operand = self.gen_expr(expr)?.load(self);

                    // Type check and type cast
                    let operand = operand.conv(ty.clone(), self)?;

                    // Assign operand to value
                    val.assign(self, operand)?;
                };
                Ok(())
            }
            Decl::Func(_, _, _) => todo!(),
            Decl::Enum(_, _) => todo!(),
            Decl::Union(_, _) => todo!(),
            Decl::Struct(_, _) => todo!(),
        }
    }

    /// Generate a expression as a statement into the program
    /// `exit`: previous exit node of the function
    /// Returns: calculated variable, new exit node of the function
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
            // Some memcpy operation is required to process arrays
            Expr::Pack(_) => todo!(),
            Expr::Map(_) => todo!(),
            Expr::Index(x, v) => {
                // Generate arguments
                let ix = self.gen_expr(v)?.load(self);
                self.gen_expr(x)?
                    .getelementptr(self, vec![Constant::Int(0).into(), ix])
            }
            Expr::Field(_, _) => todo!(),
            Expr::Select(_, _) => todo!(),
            Expr::Int32(x) => Ok(Constant::Int(*x).into()),
            Expr::Float32(x) => Ok(Constant::Float(*x).into()),
            Expr::String(_) => todo!(),
            Expr::Char(_) => todo!(),
            Expr::Bool(_) => todo!(),
            Expr::Call(_, _) => todo!(),
            Expr::Unary(_, _) => todo!(),
            Expr::Binary(op, lhs, rhs) => self.gen_binary(op, lhs, rhs),
            Expr::Conditional(_, _, _) => todo!(),
        }
    }

    /// Generate a binary expression
    pub fn gen_binary(&mut self, op: &BinaryOp, lhs: &Expr, rhs: &Expr) -> Result<Value, MiddelError> {
        // Generate arguments
        let lop = self.gen_expr(lhs)?.load(self);
        let rop = self.gen_expr(rhs)?.load(self);

        // Apply operation
        match op {
            BinaryOp::Add => {
                // Unify operand types
                let (lop, rop) = Operand::unify(lop, rop, self)?;
                let ty = lop.get_type();

                // Add "add" instruction, operand is the result of the instruction
                match ty {
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
                // Unify operand types
                let (lop, rop) = Operand::unify(lop, rop, self)?;
                let ty = lop.get_type();

                // Add "sub" instruction, operand is the result of the instruction
                match ty {
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
                // Unify operand types
                let (lop, rop) = Operand::unify(lop, rop, self)?;
                let ty = lop.get_type();

                // Add "mul" instruction, operand is the result of the instruction
                match ty {
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
                // Unify operand types
                let (lop, rop) = Operand::unify(lop, rop, self)?;
                let ty = lop.get_type();

                // Add "div" instruction, operand is the result of the instruction
                match ty {
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
                // Convert operands to int
                let lop = lop.conv(ValueType::Int, self)?;
                let rop = rop.conv(ValueType::Int, self)?;

                // Add "srem" instruction, operand is the result of the instruction
                let inst = self.program.mem_pool.get_srem(lop, rop);
                self.exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
            // Bitwise operation on int is not required
            BinaryOp::Shr => todo!(),
            BinaryOp::Shl => todo!(),
            BinaryOp::And => todo!(),
            BinaryOp::Or => todo!(),
            BinaryOp::Xor => todo!(),
            BinaryOp::Gt => {
                // Unify operand types
                let (lop, rop) = Operand::unify(lop, rop, self)?;
                let ty = lop.get_type();

                // Add "icmp" instruction, operand is the result of the instruction
                let inst =
                    self.program
                        .mem_pool
                        .get_icmp(ICmpOp::Sgt, ty, lop, rop);
                self.exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
            BinaryOp::Lt => {
                // Unify operand types
                let (lop, rop) = Operand::unify(lop, rop, self)?;
                let ty = lop.get_type();

                // Add "icmp" instruction, operand is the result of the instruction
                let inst =
                    self.program
                        .mem_pool
                        .get_icmp(ICmpOp::Slt, ty, lop, rop);
                self.exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
            BinaryOp::Ge => {
                // Unify operand types
                let (lop, rop) = Operand::unify(lop, rop, self)?;
                let ty = lop.get_type();

                // Add "icmp" instruction, operand is the result of the instruction
                let inst =
                    self.program
                        .mem_pool
                        .get_icmp(ICmpOp::Sge, ty, lop, rop);
                self.exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
            BinaryOp::Le => {
                // Unify operand types
                let (lop, rop) = Operand::unify(lop, rop, self)?;
                let ty = lop.get_type();

                // Add "icmp" instruction, operand is the result of the instruction
                let inst =
                    self.program
                        .mem_pool
                        .get_icmp(ICmpOp::Sle, ty, lop, rop);
                self.exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
            BinaryOp::Eq => {
                // Unify operand types
                let (lop, rop) = Operand::unify(lop, rop, self)?;
                let ty = lop.get_type();

                // Add "icmp" instruction, operand is the result of the instruction
                let inst =
                    self.program
                        .mem_pool
                        .get_icmp(ICmpOp::Eq, ty, lop, rop);
                self.exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
            BinaryOp::Ne => {
                // Unify operand types
                let (lop, rop) = Operand::unify(lop, rop, self)?;
                let ty = lop.get_type();

                // Add "icmp" instruction, operand is the result of the instruction
                let inst =
                    self.program
                        .mem_pool
                        .get_icmp(ICmpOp::Ne, ty, lop, rop);
                self.exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
            BinaryOp::All => {
                // Convert operands to bool
                let lop = lop.conv(ValueType::Bool, self)?;
                let rop = rop.conv(ValueType::Bool, self)?;

                // Add "and" instruction, operand is the result of the instruction
                let inst = self.program.mem_pool.get_and(lop, rop);
                self.exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
            BinaryOp::Any => {
                // Convert operands to bool
                let lop = lop.conv(ValueType::Bool, self)?;
                let rop = rop.conv(ValueType::Bool, self)?;

                // Add "or" instruction, operand is the result of the instruction
                let inst = self.program.mem_pool.get_or(lop, rop);
                self.exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
        }
    }
}
