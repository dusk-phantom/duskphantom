use std::collections::HashMap;

use crate::{
    errors::MiddelError,
    frontend::{self, BinaryOp, Decl, Expr, Stmt, Type},
    middle,
};

use super::ir::{instruction::misc_inst::ICmpOp, BBPtr, Constant, FunPtr, Operand, ValueType};

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

/// Convenient methods for operand
impl Operand {
    /// Convert the type of an operand to another
    fn conv<'a>(self, ty: ValueType, kit: &mut FunctionKit<'a>) -> Result<Operand, MiddelError> {
        let from_ty = self.get_type();
        if from_ty == ty {
            return Ok(self);
        }
        match (from_ty, ty) {
            (ValueType::Int, ValueType::Float) => {
                // Direct convert
                let inst = kit.program.mem_pool.get_itofp(self);
                kit.exit.push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Float, ValueType::Int) => {
                // Direct convert
                let inst = kit.program.mem_pool.get_fptoi(self);
                kit.exit.push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Bool, ValueType::Int) => {
                // Direct convert
                let inst = kit.program.mem_pool.get_zext(self);
                kit.exit.push_back(inst);
                Ok(inst.into())
            }
            (ValueType::Bool, ValueType::Float) => {
                // Convert to int first and then float
                let inst = kit.program.mem_pool.get_zext(self);
                let inst = kit.program.mem_pool.get_itofp(inst.into());
                kit.exit.push_back(inst);
                Ok(inst.into())
            }
            _ => Err(MiddelError::GenError),
        }
    }

    /// Unify the types of two operands
    fn unify<'a>(a: Self, b: Self, kit: &mut FunctionKit<'a>) -> Result<(Self, Self), MiddelError> {
        let a_ty = a.get_type();
        let b_ty = b.get_type();
        let max_ty = a_ty.max_with(&b_ty)?;
        Ok((a.conv(max_ty.clone(), kit)?, b.conv(max_ty, kit)?))
    }
}

/// Convenient methods for value type
impl ValueType {
    /// If a value type can be converted to a number, returns true
    fn is_num(&self) -> bool {
        match self {
            ValueType::Bool => true,
            ValueType::Int => true,
            ValueType::Float => true,
            _ => false,
        }
    }

    /// Convert a numeric value type to its percision level
    /// Higher is more percise
    fn to_percision_level(&self) -> i32 {
        match self {
            ValueType::Bool => 1,
            ValueType::Int => 2,
            ValueType::Float => 3,
            _ => 0,
        }
    }

    /// Convert a percision level to a value type
    fn from_percision_level(level: i32) -> Self {
        match level {
            1 => ValueType::Bool,
            2 => ValueType::Int,
            3 => ValueType::Float,
            _ => ValueType::Void,
        }
    }

    /// Max this type with another type
    /// Return more precise one
    fn max_with(&self, b: &Self) -> Result<Self, MiddelError> {
        if self.is_num() && b.is_num() {
            let a_lv = self.to_percision_level();
            let b_lv = b.to_percision_level();
            let max_lv = if a_lv > b_lv { a_lv } else { b_lv };
            Ok(ValueType::from_percision_level(max_lv))
        } else {
            Err(MiddelError::GenError)
        }
    }
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

/// A value can be an operand, or a pointer to an operand
/// An operand can not be assigned to, while a pointed value can
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

/// A constant can be converted to a value
impl Into<Value> for Constant {
    fn into(self) -> Value {
        Value::Operand(Operand::Constant(self))
    }
}

/// Convenient operations on a value
impl Value {
    /// Get the type of a value
    fn get_type(&self) -> ValueType {
        match self {
            Value::Operand(op) => op.get_type(),
            Value::Pointer(op) => match op.get_type() {
                // Inside `Pointer` is the pointer to given value
                ValueType::Pointer(ty) => *ty,
                _ => todo!(),
            },
        }
    }

    /// Load the value as an operand
    fn load<'a>(self, kit: &mut FunctionKit<'a>) -> Operand {
        let ty = self.get_type();
        match self {
            Value::Operand(op) => op,
            Value::Pointer(op) => {
                // Add instruction to exit
                let inst = kit.program.mem_pool.get_load(ty, op);
                kit.exit.push_back(inst);
                inst.into()
            }
        }
    }

    /// Shift the underlying pointer (if exists)
    /// Element of index is [shift by whole, shift by primary element, ...]
    /// For example, getelementptr([2, 8]) on a pointer to an array [n x i32]
    /// shifts it by (2 * n + 8) * sizeof i32.
    /// DO NOT FORGET THE FIRST INDEX
    fn getelementptr<'a>(
        self,
        kit: &mut FunctionKit<'a>,
        index: Vec<Operand>,
    ) -> Result<Value, MiddelError> {
        let ty = self.get_type();
        match self {
            Value::Operand(op) => Err(MiddelError::GenError),
            Value::Pointer(op) => {
                // Add instruction to exit
                let inst = kit.program.mem_pool.get_getelementptr(ty, op, index);
                kit.exit.push_back(inst);

                // Construct new value
                // TODO Type of pointer is shrinked (as "get element" states)
                Ok(Value::Pointer(inst.into()))
            }
        }
    }

    /// Assign an operand to this value
    fn assign<'a>(self, kit: &mut FunctionKit<'a>, op: Operand) -> Result<(), MiddelError> {
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

/// Kit for translating a program to middle IR
struct ProgramKit<'a> {
    env: HashMap<String, Value>,
    fenv: HashMap<String, FunPtr>,
    ctx: HashMap<String, ValueType>,
    program: &'a mut middle::Program,
}

/// Kit for translating a function to middle IR
struct FunctionKit<'a> {
    env: HashMap<String, Value>,
    fenv: HashMap<String, FunPtr>,
    ctx: HashMap<String, ValueType>,
    program: &'a mut middle::Program,
    entry: BBPtr,
    exit: BBPtr,
    break_to: Option<BBPtr>,
    continue_to: Option<BBPtr>,
    return_type: ValueType,
}

/// Repeat a vector for `n` times
fn repeat_vec<T>(vec: Vec<T>, n: usize) -> Vec<T>
where
    T: Clone,
{
    let mut result = Vec::new();
    for _ in 0..n {
        result.extend(vec.clone());
    }
    result
}

/// Convert a type to its default constant
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

/// Convert a constant expression to a constant
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
    fn gen(mut self, program: &frontend::Program) -> Result<(), MiddelError> {
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
                        // This global variable is mutable
                        true,
                        expr_to_const(v)?,
                    ),
                    None => self.program.mem_pool.new_global_variable(
                        id.clone(),
                        translate_type(ty),
                        true,
                        type_to_const(ty)?,
                    ),
                };

                // Add global variable (pointer) to environment
                self.env.insert(id.clone(), Value::Pointer(gval.into()));

                // Add global variable to program
                self.program.module.global_variables.push(gval);
                Ok(())
            }
            Decl::Func(ty, id, op) => {
                if let (Some(stmt), Type::Function(return_ty, params)) = (op, ty) {
                    // Get function type
                    let fty = translate_type(return_ty);

                    // Create function
                    let mut fptr = self.program.mem_pool.new_function(id.clone(), fty.clone());

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
                        return_type: fty,
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
                    fenv: self.fenv.clone(),
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
                    fenv: self.fenv.clone(),
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
                    fenv: self.fenv.clone(),
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
                    fenv: self.fenv.clone(),
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
    fn gen_binary(&mut self, op: &BinaryOp, lhs: &Box<Expr>, rhs: &Box<Expr>) -> Result<Value, MiddelError> {
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
        // No constant folding, because a variable can be re-assigned in SysY
        // This behaviour is consistent with `clang -S -emit-llvm xxx.c`
        assert_eq!(
            llvm_ir,
            "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 1, ptr %alloca_1\n%alloca_3 = alloca i32\nstore i32 2, ptr %alloca_3\n%alloca_5 = alloca i32\n%load_6 = load i32, ptr %alloca_1\n%load_7 = load i32, ptr %alloca_3\n%Add_8 = add i32, %load_6, %load_7\nstore i32 %Add_8, ptr %alloca_5\n%load_10 = load i32, ptr %alloca_5\nret %load_10\n\n\n}\n"
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
            "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 1, ptr %alloca_1\n%alloca_3 = alloca i32\nstore i32 2, ptr %alloca_3\nbr label %cond\n\n%cond:\n%load_10 = load i32, ptr %alloca_1\n%load_11 = load i32, ptr %alloca_3\n%icmp_12 = icmp slt i32 %load_10, %load_11\nbr i1 %icmp_12, label %then, label %alt\n\n%then:\nstore i32 3, ptr %alloca_1\nbr label %final\n\n%alt:\nstore i32 4, ptr %alloca_1\nbr label %final\n\n%final:\n%load_18 = load i32, ptr %alloca_1\nret %load_18\n\n\n}\n"
        );
    }

    #[test]
    fn test_while() {
        let code = r#"
            int main() {
                int a = 0;
                while (a < 10) {
                    a = a + 1;
                }
                return a;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(0)))), While(Binary(Lt, Var(\"a\"), Int32(10)), Block([Expr(Some(Var(\"a\")), Binary(Add, Var(\"a\"), Int32(1)))])), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 0, ptr %alloca_1\nbr label %cond\n\n%cond:\n%load_7 = load i32, ptr %alloca_1\n%icmp_8 = icmp slt i32 %load_7, 10\nbr i1 %icmp_8, label %body, label %final\n\n%body:\n%load_10 = load i32, ptr %alloca_1\n%Add_11 = add i32, %load_10, 1\nstore i32 %Add_11, ptr %alloca_1\nbr label %cond\n\n%final:\n%load_14 = load i32, ptr %alloca_1\nret %load_14\n\n\n}\n");
    }

    #[test]
    fn test_do_while() {
        let code = r#"
            int main() {
                int a = 0;
                do {
                    a = a + 1;
                } while (a < 10);
                return a;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(0)))), DoWhile(Block([Expr(Some(Var(\"a\")), Binary(Add, Var(\"a\"), Int32(1)))]), Binary(Lt, Var(\"a\"), Int32(10))), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 0, ptr %alloca_1\nbr label %body\n\n%body:\n%load_10 = load i32, ptr %alloca_1\n%Add_11 = add i32, %load_10, 1\nstore i32 %Add_11, ptr %alloca_1\nbr label %cond\n\n%cond:\n%load_7 = load i32, ptr %alloca_1\n%icmp_8 = icmp slt i32 %load_7, 10\nbr i1 %icmp_8, label %body, label %final\n\n%final:\n%load_14 = load i32, ptr %alloca_1\nret %load_14\n\n\n}\n");
    }

    #[test]
    fn test_break() {
        let code = r#"
            int main() {
                int a = 0;
                while (a < 10) {
                    a = a + 1;
                    break;
                }
                return a;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(0)))), While(Binary(Lt, Var(\"a\"), Int32(10)), Block([Expr(Some(Var(\"a\")), Binary(Add, Var(\"a\"), Int32(1))), Break])), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        // There are two `br` in `%body` block
        // Not preventing this can make `irgen` code simpler
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 0, ptr %alloca_1\nbr label %cond\n\n%cond:\n%load_7 = load i32, ptr %alloca_1\n%icmp_8 = icmp slt i32 %load_7, 10\nbr i1 %icmp_8, label %body, label %final\n\n%body:\n%load_10 = load i32, ptr %alloca_1\n%Add_11 = add i32, %load_10, 1\nstore i32 %Add_11, ptr %alloca_1\nbr label %final\nbr label %final\n\n%final:\n%load_15 = load i32, ptr %alloca_1\nret %load_15\n\n\n}\n");
    }

    #[test]
    fn test_global_variable() {
        let code = r#"
            int x = 4;
            int y = 8;
            int main() {
                x = x + y;
                return x;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Var(Int32, \"x\", Some(Int32(4))), Var(Int32, \"y\", Some(Int32(8))), Func(Function(Int32, []), \"main\", Some(Block([Expr(Some(Var(\"x\")), Binary(Add, Var(\"x\"), Var(\"y\"))), Return(Some(Var(\"x\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "@x = dso_local global i32 [4]\n@y = dso_local global i32 [8]\ndefine i32 @main() {\n%entry:\n%load_1 = load i32, ptr @x\n%load_2 = load i32, ptr @y\n%Add_3 = add i32, %load_1, %load_2\nstore i32 %Add_3, ptr @x\n%load_5 = load i32, ptr @x\nret %load_5\n\n\n}\n");
    }

    #[test]
    fn test_conv() {
        let code = r#"
            int main() {
                int x = 1;
                float y = 2.0;
                float z = x + y;
                return z;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"x\", Some(Int32(1)))), Decl(Var(Float32, \"y\", Some(Float32(2.0)))), Decl(Var(Float32, \"z\", Some(Binary(Add, Var(\"x\"), Var(\"y\"))))), Return(Some(Var(\"z\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 1, ptr %alloca_1\n%alloca_3 = alloca float\nstore float 2, ptr %alloca_3\n%alloca_5 = alloca float\n%load_6 = load i32, ptr %alloca_1\n%load_7 = load float, ptr %alloca_3\n%itofp_8 = sitofp i32 %load_6 to float\n%FAdd_9 = fadd float, %itofp_8, %load_7\nstore float %FAdd_9, ptr %alloca_5\n%load_11 = load float, ptr %alloca_5\n%fptoi_12 = fptosi float %load_11 to i32\nret %fptoi_12\n\n\n}\n");
    }

    // #[test]
    // fn test_template() {
    //     let code = r#"
    //         int main() {
    //             int a = 0;
    //             while (a < 10) {
    //                 a = a + 1;
    //             }
    //             return a;
    //         }
    //     "#;
    //     let program = parse(code).unwrap();
    //     assert_eq!(format!("{:?}", program), "");
    //     let result = gen(&program).unwrap();
    //     let llvm_ir = result.module.gen_llvm_ir();
    //     assert_eq!(llvm_ir, "");
    // }
}
