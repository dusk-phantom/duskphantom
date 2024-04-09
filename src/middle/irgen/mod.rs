use std::collections::HashMap;

use crate::{errors::MiddelError, frontend, middle, middle::ir};

/// Generate middle IR from a frontend AST
pub fn gen(program: &frontend::Program) -> Result<middle::Program, MiddelError> {
    let mut result = middle::Program::new();
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
    program: &'a mut middle::Program,
}

/// Kit for translating a function to middle IR.
struct FunctionKit<'a> {
    env: HashMap<String, ir::Operand>,
    fenv: HashMap<String, ir::FunPtr>,
    ctx: HashMap<String, ir::ValueType>,
    program: &'a mut middle::Program,
    entry: ir::BBPtr,
    exit: ir::BBPtr,
    break_to: Option<ir::BBPtr>,
    continue_to: Option<ir::BBPtr>,
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

                // Add global variable to program
                self.program.module.global_variables.push(gval);
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

                    // Build function
                    let fname = self.unique_debug("entry");
                    let bb = self.program.mem_pool.new_basicblock(fname);
                    let mut kit = FunctionKit {
                        program: self.program,
                        env: self.env.clone(),
                        fenv: self.fenv.clone(),
                        ctx: self.ctx.clone(),
                        entry: bb,
                        exit: bb,
                        break_to: None,
                        continue_to: None,
                    };
                    kit.gen_stmt(stmt)?;
                    fptr.entry = Some(kit.entry);
                    fptr.exit = Some(kit.exit);

                    // Add function to environment
                    self.fenv.insert(id.clone(), fptr);

                    // Add function to programs
                    self.program.module.functions.push(fptr);
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
                let operand = self.gen_expr(cond)?;
                let ir::Operand::Instruction(inst) = operand else {
                    todo!("make get_br accept operand")
                };
                let br = self.program.mem_pool.get_br(Some(inst));
                cond_bb.push_back(br);

                // Add statements and br to then branch
                FunctionKit {
                    program: self.program,
                    env: self.env.clone(),
                    fenv: self.fenv.clone(),
                    ctx: self.ctx.clone(),
                    entry: then_bb,
                    exit: then_bb,
                    break_to: None,
                    continue_to: None,
                }
                .gen_stmt(then)?;
                then_bb.push_back(self.program.mem_pool.get_br(None));

                // Add statements and br to alt branch
                FunctionKit {
                    program: self.program,
                    env: self.env.clone(),
                    fenv: self.fenv.clone(),
                    ctx: self.ctx.clone(),
                    entry: alt_bb,
                    exit: alt_bb,
                    break_to: None,
                    continue_to: None,
                }
                .gen_stmt(alt)?;
                alt_bb.push_back(self.program.mem_pool.get_br(None));

                // Increment exit
                self.exit = final_bb;
                Ok(())
            }
            frontend::Stmt::While(cond, body) => {
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
                let operand = self.gen_expr(cond)?;
                let ir::Operand::Instruction(inst) = operand else {
                    todo!("make get_br accept operand")
                };
                let br = self.program.mem_pool.get_br(Some(inst));
                cond_bb.push_back(br);

                // Add statements and br to body block
                FunctionKit {
                    program: self.program,
                    env: self.env.clone(),
                    fenv: self.fenv.clone(),
                    ctx: self.ctx.clone(),
                    entry: body_bb,
                    exit: body_bb,
                    break_to: Some(final_bb),
                    continue_to: Some(cond_bb),
                }.gen_stmt(body)?;
                body_bb.push_back(self.program.mem_pool.get_br(None));

                // Increment exit
                self.exit = final_bb;
                Ok(())
            },
            frontend::Stmt::DoWhile(body, cond) => {
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
                let operand = self.gen_expr(cond)?;
                let ir::Operand::Instruction(inst) = operand else {
                    todo!("make get_br accept operand")
                };
                let br = self.program.mem_pool.get_br(Some(inst));
                cond_bb.push_back(br);

                // Add statements and br to body block
                FunctionKit {
                    program: self.program,
                    env: self.env.clone(),
                    fenv: self.fenv.clone(),
                    ctx: self.ctx.clone(),
                    entry: body_bb,
                    exit: body_bb,
                    break_to: Some(final_bb),
                    continue_to: Some(cond_bb),
                }.gen_stmt(body)?; 
                body_bb.push_back(self.program.mem_pool.get_br(None));

                // Increment exit
                self.exit = final_bb;
                Ok(())
            },
            frontend::Stmt::For(_, _, _, _) => todo!(),
            frontend::Stmt::Break => {
                // Add br instruction to exit block
                // TODO make condition constant False
                let br = self.program.mem_pool.get_br(None);
                self.exit.push_back(br);

                // When break statement appears, break_to must not be None
                let Some(break_to) = self.break_to else {
                    return Err(MiddelError::GenError);
                };

                // A break can only appear in non-condition block
                // So it is safe to set false branch to break_to
                self.exit.set_false_bb(break_to);
                Ok(())
            },
            frontend::Stmt::Continue => {
                // Add br instruction to exit block
                // TODO make condition constant False
                let br = self.program.mem_pool.get_br(None);
                self.exit.push_back(br);

                // When continue statement appears, continue_to must not be None
                let Some(continue_to) = self.continue_to else {
                    return Err(MiddelError::GenError);
                };

                // A continue can only appear in non-condition block
                // So it is safe to set false branch to continue_to
                self.exit.set_false_bb(continue_to);
                Ok(())
            },
            frontend::Stmt::Return(expr) => {
                // Add returned result to exit block
                let return_value = match expr {
                    Some(expr) => {
                        let operand = self.gen_expr(expr)?;
                        let ir::Operand::Instruction(inst) = operand else {
                            todo!("make get_br accept operand")
                        };
                        Some(inst)
                    }
                    None => None
                };

                // Add ret instruction to exit block
                let ret = self.program.mem_pool.get_ret(return_value);
                self.exit.push_back(ret);
                Ok(())
            },
            frontend::Stmt::Block(stmts) => {
                // Add statements to current block
                for stmt in stmts.iter() {
                    self.gen_stmt(stmt)?;
                }
                Ok(())
            },
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
            // Some memcpy operation is required to process arrays
            frontend::Expr::Pack(_) => todo!(),
            frontend::Expr::Map(_) => todo!(),
            frontend::Expr::Index(_, _) => todo!(),
            frontend::Expr::Field(_, _) => todo!(),
            frontend::Expr::Select(_, _) => todo!(),
            frontend::Expr::Int32(x) => Ok(ir::Operand::Constant(ir::Constant::Int(*x))),
            frontend::Expr::Float32(x) => Ok(ir::Operand::Constant(ir::Constant::Float(*x))),
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
                        let inst = self.program.mem_pool.get_add(lop, rop);
                        self.exit.push_back(inst);
                        Ok(ir::Operand::Instruction(inst))
                    }
                    frontend::BinaryOp::Sub => {
                        // Add "inst" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_sub(lop, rop);
                        self.exit.push_back(inst);
                        Ok(ir::Operand::Instruction(inst))
                    }
                    frontend::BinaryOp::Mul => {
                        // Add "inst" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_mul(lop, rop);
                        self.exit.push_back(inst);
                        Ok(ir::Operand::Instruction(inst))
                    }
                    frontend::BinaryOp::Div => {
                        // Add "inst" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_sdiv(lop, rop);
                        self.exit.push_back(inst);
                        Ok(ir::Operand::Instruction(inst))
                    }
                    frontend::BinaryOp::Mod => {
                        // Add "inst" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_srem(lop, rop);
                        self.exit.push_back(inst);
                        Ok(ir::Operand::Instruction(inst))
                    }
                    // Bitwise operation on int is not required
                    frontend::BinaryOp::Shr => todo!(),
                    frontend::BinaryOp::Shl => todo!(),
                    frontend::BinaryOp::And => todo!(),
                    frontend::BinaryOp::Or => todo!(),
                    frontend::BinaryOp::Xor => todo!(),
                    frontend::BinaryOp::Gt => {
                        // Add "icmp" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_icmp(
                            ir::instruction::misc_inst::ICmpOp::Sgt, 
                            ir::ValueType::Int, 
                            lop, 
                            rop,
                        );
                        self.exit.push_back(inst);
                        Ok(ir::Operand::Instruction(inst))
                    }
                    frontend::BinaryOp::Lt => {
                        // Add "icmp" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_icmp(
                            ir::instruction::misc_inst::ICmpOp::Slt, 
                            ir::ValueType::Int, 
                            lop, 
                            rop,
                        );
                        self.exit.push_back(inst);
                        Ok(ir::Operand::Instruction(inst))
                    }
                    frontend::BinaryOp::Ge => {
                        // Add "icmp" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_icmp(
                            ir::instruction::misc_inst::ICmpOp::Sge, 
                            ir::ValueType::Int, 
                            lop, 
                            rop,
                        );
                        self.exit.push_back(inst);
                        Ok(ir::Operand::Instruction(inst))
                    }
                    frontend::BinaryOp::Le => {
                        // Add "icmp" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_icmp(
                            ir::instruction::misc_inst::ICmpOp::Sle, 
                            ir::ValueType::Int, 
                            lop, 
                            rop,
                        );
                        self.exit.push_back(inst);
                        Ok(ir::Operand::Instruction(inst))
                    }
                    frontend::BinaryOp::Eq => {
                        // Add "icmp" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_icmp(
                            ir::instruction::misc_inst::ICmpOp::Eq, 
                            ir::ValueType::Int, 
                            lop, 
                            rop,
                        );
                        self.exit.push_back(inst);
                        Ok(ir::Operand::Instruction(inst))
                    }
                    frontend::BinaryOp::Ne => {
                        // Add "icmp" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_icmp(
                            ir::instruction::misc_inst::ICmpOp::Ne, 
                            ir::ValueType::Int, 
                            lop, 
                            rop,
                        );
                        self.exit.push_back(inst);
                        Ok(ir::Operand::Instruction(inst))
                    }
                    frontend::BinaryOp::All => {
                        // Add "and" instruction, operand is the result of the instruction
                        let and = self.program.mem_pool.get_and(lop, rop);
                        self.exit.push_back(and);
                        Ok(ir::Operand::Instruction(and))
                    }
                    frontend::BinaryOp::Any => {
                        // Add "or" instruction, operand is the result of the instruction
                        let or = self.program.mem_pool.get_or(lop, rop);
                        self.exit.push_back(or);
                        Ok(ir::Operand::Instruction(or))
                    }
                }
            }
            frontend::Expr::Conditional(_, _, _) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::program::parse;

    #[test]
    fn test_gen() {
        let code = r#"
            int main() {
                int a = 1;
                int b = 2;
                int c = a + b;
                return c;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(1)))), Decl(Var(Int32, \"b\", Some(Int32(2)))), Decl(Var(Int32, \"c\", Some(Binary(Add, Var(\"a\"), Var(\"b\"))))), Return(Some(Var(\"c\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "n() {\n%entry:\n%Add_1 = add i32, 1, 2ret %Add_1\n\n}\n");
    }
}