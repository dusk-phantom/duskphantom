use std::collections::HashMap;

use crate::{
    errors::MiddelError,
    frontend::{self, BinaryOp, Decl, Expr, Stmt, Type},
    middle,
};

use super::ir::{
    instruction::misc_inst::ICmpOp, BBPtr, Constant, FunPtr, GlobalPtr, Operand, ValueType,
};

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
fn translate_type(ty: &Type) -> ValueType {
    match ty {
        Type::Void => ValueType::Void,
        Type::Int32 => ValueType::Int,
        Type::Float32 => ValueType::Float,
        Type::String => todo!(),
        Type::Char => todo!(),
        Type::Boolean => ValueType::Bool,
        Type::Pointer(ty) => ValueType::Pointer(Box::new(translate_type(ty))),
        Type::Array(ty, n) => ValueType::Array(Box::new(translate_type(ty)), *n),
        Type::Function(_, _) => todo!(),
        Type::Enum(_) => todo!(),
        Type::Union(_) => todo!(),
        Type::Struct(_) => todo!(),
    }
}

/// A value can be an operand, or a pointer to an operand.
/// An operand can not be assigned to, while a pointed value can.
#[derive(Clone)]
enum Value {
    Operand(Operand),
    Pointer(Operand),
}

/// A value can be allocated with type and kit
fn alloc<'a>(ty: ValueType, kit: &mut FunctionKit<'a>) -> Value {
    // Add instruction to exit
    let inst = kit.program.mem_pool.get_alloca(ty, 1);
    kit.exit.push_back(inst);
    Value::Pointer(inst.into())
}

/// TODO put this code to global ptr
/// A global variable can be converted to an operand
impl Into<Operand> for GlobalPtr {
    fn into(self) -> Operand {
        Operand::Global(self)
    }
}

/// A constant can be converted to a value
impl Into<Value> for Constant {
    fn into(self) -> Value {
        Value::Operand(Operand::Constant(self))
    }
}

/// A value can be loaded or assigned, and has type
impl Value {
    pub fn get_type(&self) -> ValueType {
        match self {
            Value::Operand(op) => op.get_type(),
            Value::Pointer(op) => match op.get_type() {
                // Inside `Pointer` is the pointer to given value
                ValueType::Pointer(ty) => *ty,
                _ => todo!(),
            },
        }
    }

    pub fn load<'a>(self, kit: &mut FunctionKit<'a>) -> Operand {
        match self {
            Value::Operand(op) => op,
            Value::Pointer(op) => {
                // Add instruction to exit
                let inst = kit.program.mem_pool.get_load(op.get_type(), op);
                kit.exit.push_back(inst);
                inst.into()
            }
        }
    }

    pub fn assign<'a>(self, kit: &mut FunctionKit<'a>, op: Operand) -> Result<(), MiddelError> {
        match self {
            Value::Operand(_) => Err(MiddelError::GenError),
            Value::Pointer(ptr) => {
                // Add instruction to exit
                let inst = kit.program.mem_pool.get_store(op, ptr);
                kit.exit.push_back(inst);
                Ok(())
            }
        }
    }
}

/// Kit for translating a program to middle IR.
struct ProgramKit<'a> {
    env: HashMap<String, Value>,
    fenv: HashMap<String, FunPtr>,
    ctx: HashMap<String, ValueType>,
    program: &'a mut middle::Program,
}

/// Kit for translating a function to middle IR.
struct FunctionKit<'a> {
    env: HashMap<String, Value>,
    fenv: HashMap<String, FunPtr>,
    ctx: HashMap<String, ValueType>,
    program: &'a mut middle::Program,
    entry: BBPtr,
    exit: BBPtr,
    break_to: Option<BBPtr>,
    continue_to: Option<BBPtr>,
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

fn type_to_const(ty: &Type) -> Result<Vec<Constant>, MiddelError> {
    match ty {
        Type::Void => todo!(),
        Type::Int32 => Ok(vec![Constant::Int(0)]),
        Type::Float32 => Ok(vec![Constant::Float(0.0)]),
        Type::String => todo!(),
        Type::Char => todo!(),
        Type::Boolean => Ok(vec![Constant::Bool(false)]),
        Type::Pointer(_) => todo!(),
        Type::Array(ty, num) => Ok(repeat_vec(type_to_const(ty)?, *num)),
        Type::Function(_, _) => Err(MiddelError::GenError),
        Type::Enum(_) => todo!(),
        Type::Union(_) => todo!(),
        Type::Struct(_) => todo!(),
    }
}

fn expr_to_const(val: &Expr) -> Result<Vec<Constant>, MiddelError> {
    match val {
        Expr::Var(_) => todo!(),
        Expr::Pack(pack) => pack
            .iter()
            // Convert inner expression to constant value
            .map(expr_to_const)
            // Collect as a large result
            .collect::<Result<Vec<Vec<_>>, _>>()
            // Flatten inner vec
            .map(|v| v.into_iter().flatten().collect()),
        Expr::Map(_) => todo!(),
        Expr::Index(_, _) => todo!(),
        Expr::Field(_, _) => todo!(),
        Expr::Select(_, _) => todo!(),
        Expr::Int32(i) => Ok(vec![Constant::Int(*i)]),
        Expr::Float32(f) => Ok(vec![Constant::Float(*f)]),
        Expr::String(_) => todo!(),
        Expr::Char(_) => todo!(),
        Expr::Bool(b) => Ok(vec![Constant::Bool(*b)]),
        Expr::Call(_, _) => todo!(),
        Expr::Unary(_, _) => todo!(),
        Expr::Binary(_, _, _) => todo!(),
        Expr::Conditional(_, _, _) => todo!(),
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
    fn gen_decl(&mut self, decl: &Decl) -> Result<(), MiddelError> {
        match decl {
            Decl::Var(ty, id, val) => {
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
                // TODO how to set global variable?
                self.env.insert(id.clone(), Value::Operand(gval.into()));

                // Add global variable to program
                self.program.module.global_variables.push(gval);
                Ok(())
            }
            Decl::Func(ty, id, op) => {
                if let (Some(stmt), Type::Function(return_ty, params)) = (op, ty) {
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
            Decl::Enum(_, _) => todo!(),
            Decl::Union(_, _) => todo!(),
            Decl::Struct(_, _) => todo!(),
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
    fn gen_stmt(&mut self, stmt: &Stmt) -> Result<(), MiddelError> {
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
                            // Make sure variable can be assigned
                            let Some(val @ Value::Pointer(_)) = self.env.get(id) else {
                                return Err(MiddelError::GenError);
                            };

                            // Typecheck, TODO type cast
                            if operand.get_type() != val.get_type() {
                                return Err(MiddelError::GenError);
                            }

                            // Assign to value
                            val.clone().assign(self, operand)?;
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
                let value = self.gen_expr(cond)?;
                let Operand::Instruction(inst) = value.load(self) else {
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
                let value = self.gen_expr(cond)?;
                let Operand::Instruction(inst) = value.load(self) else {
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
                let value = self.gen_expr(cond)?;
                let Operand::Instruction(inst) = value.load(self) else {
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
            }
            Stmt::Continue => {
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
            }
            Stmt::Return(expr) => {
                // Add returned result to exit block
                let return_value = match expr {
                    Some(expr) => {
                        let value = self.gen_expr(expr)?;
                        let Operand::Instruction(inst) = value.load(self) else {
                            todo!("make get_br accept operand")
                        };
                        Some(inst)
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
    fn gen_decl(&mut self, decl: &Decl) -> Result<(), MiddelError> {
        match decl {
            Decl::Var(raw_ty, id, op) => {
                // Add type to context
                let ty = translate_type(raw_ty);
                self.ctx.insert(id.clone(), ty.clone());

                // Allocate space for variable, add to environment
                let val = alloc(ty.clone(), self);
                self.env.insert(id.clone(), val.clone());

                // Assign to the variable if it is defined
                if let Some(expr) = op {
                    // Generate expression
                    let operand = self.gen_expr(expr)?.load(self);

                    // Typecheck, TODO type cast
                    if operand.get_type() != ty {
                        return Err(MiddelError::GenError);
                    }

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
    fn gen_expr(&mut self, expr: &Expr) -> Result<Value, MiddelError> {
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
            Expr::Index(_, _) => todo!(),
            Expr::Field(_, _) => todo!(),
            Expr::Select(_, _) => todo!(),
            Expr::Int32(x) => Ok(Constant::Int(*x).into()),
            Expr::Float32(x) => Ok(Constant::Float(*x).into()),
            Expr::String(_) => todo!(),
            Expr::Char(_) => todo!(),
            Expr::Bool(_) => todo!(),
            Expr::Call(_, _) => todo!(),
            Expr::Unary(_, _) => todo!(),
            Expr::Binary(op, lhs, rhs) => {
                let lop = self.gen_expr(lhs)?.load(self);
                let rop = self.gen_expr(rhs)?.load(self);
                match op {
                    BinaryOp::Add => {
                        // Add "add" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_add(lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    BinaryOp::Sub => {
                        // Add "inst" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_sub(lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    BinaryOp::Mul => {
                        // Add "inst" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_mul(lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    BinaryOp::Div => {
                        // Add "inst" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_sdiv(lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    BinaryOp::Mod => {
                        // Add "inst" instruction, operand is the result of the instruction
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
                        // Add "icmp" instruction, operand is the result of the instruction
                        let inst =
                            self.program
                                .mem_pool
                                .get_icmp(ICmpOp::Sgt, ValueType::Int, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    BinaryOp::Lt => {
                        // Add "icmp" instruction, operand is the result of the instruction
                        let inst =
                            self.program
                                .mem_pool
                                .get_icmp(ICmpOp::Slt, ValueType::Int, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    BinaryOp::Ge => {
                        // Add "icmp" instruction, operand is the result of the instruction
                        let inst =
                            self.program
                                .mem_pool
                                .get_icmp(ICmpOp::Sge, ValueType::Int, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    BinaryOp::Le => {
                        // Add "icmp" instruction, operand is the result of the instruction
                        let inst =
                            self.program
                                .mem_pool
                                .get_icmp(ICmpOp::Sle, ValueType::Int, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    BinaryOp::Eq => {
                        // Add "icmp" instruction, operand is the result of the instruction
                        let inst =
                            self.program
                                .mem_pool
                                .get_icmp(ICmpOp::Eq, ValueType::Int, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    BinaryOp::Ne => {
                        // Add "icmp" instruction, operand is the result of the instruction
                        let inst =
                            self.program
                                .mem_pool
                                .get_icmp(ICmpOp::Ne, ValueType::Int, lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    BinaryOp::All => {
                        // Add "and" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_and(lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                    BinaryOp::Any => {
                        // Add "or" instruction, operand is the result of the instruction
                        let inst = self.program.mem_pool.get_or(lop, rop);
                        self.exit.push_back(inst);
                        Ok(Value::Operand(inst.into()))
                    }
                }
            }
            Expr::Conditional(_, _, _) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::program::parse;

    #[test]
    fn test_normal() {
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
        assert_eq!(
            llvm_ir,
            // TODO line break?
            "n() {\n%entry:\n%alloca_1 = alloca i32 i32store i32 1, ptr %alloca_1%alloca_3 = alloca i32 i32store i32 2, ptr %alloca_3%alloca_5 = alloca i32 i32%load_6 = load i32*, ptr %alloca_1%load_7 = load i32*, ptr %alloca_3%Add_8 = add i32, %load_6, %load_7store i32 %Add_8, ptr %alloca_5%load_10 = load i32*, ptr %alloca_5ret %load_10\n\n}\n"
        );
    }

    #[test]
    fn test_if() {
        let code = r#"
            int main() {
                int a = 1;
                int b = 2;
                if (a < b) {
                    a = 3;
                } else {
                    a = 4;
                }
                return a;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(1)))), Decl(Var(Int32, \"b\", Some(Int32(2)))), If(Binary(Lt, Var(\"a\"), Var(\"b\")), Block([Expr(Some(Var(\"a\")), Int32(3))]), Block([Expr(Some(Var(\"a\")), Int32(4))])), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(
            llvm_ir,
            "n() {
%entry:
%Add_1 = add i32, 1, 2ret %Add_1

}
"
        );
    }
}
