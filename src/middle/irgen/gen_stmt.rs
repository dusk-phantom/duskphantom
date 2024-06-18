use anyhow::{anyhow, Context, Result};

use crate::context;
use crate::frontend::Stmt;
use crate::middle::ir::ValueType;

use super::function_kit::FunctionKit;

impl<'a> FunctionKit<'a> {
    /// Generate a statement into the program
    pub fn gen_stmt(&mut self, stmt: &Stmt) -> Result<&Self> {
        let Some(mut exit) = self.exit else {
            return Err(anyhow!("exit block can't be appended")).with_context(|| context!());
        };
        match stmt {
            Stmt::Nothing => (),
            Stmt::Decl(decl) => {
                // Generate declaration
                self.gen_inner_decl(decl)?;
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
                exit.set_true_bb(cond_bb);
                exit.push_back(self.program.mem_pool.get_br(None));

                // Add condition and br to condition block
                self.exit = Some(cond_bb);
                let operand = self.gen_expr(cond)?.load(ValueType::Bool, self)?;
                let br = self.program.mem_pool.get_br(Some(operand));
                cond_bb.push_back(br);
                cond_bb.set_true_bb(then_entry);
                cond_bb.set_false_bb(alt_entry);

                // Add statements and br to then branch
                // Retain break_to and continue_to
                let then_exit = self
                    .gen_function_kit(then_entry, self.break_to, self.continue_to)
                    .gen_stmt(then)?
                    .exit;
                if let Some(mut then_exit) = then_exit {
                    then_exit.push_back(self.program.mem_pool.get_br(None));
                    then_exit.set_true_bb(final_bb);
                }

                // Add statements and br to alt branch
                let alt_exit = self
                    .gen_function_kit(alt_entry, self.break_to, self.continue_to)
                    .gen_stmt(alt)?
                    .exit;
                if let Some(mut alt_exit) = alt_exit {
                    alt_exit.push_back(self.program.mem_pool.get_br(None));
                    alt_exit.set_true_bb(final_bb);
                }

                // Exit is final block
                self.exit = Some(final_bb);
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
                exit.set_true_bb(cond_bb);
                exit.push_back(self.program.mem_pool.get_br(None));

                // Add statements and br to body block
                let body_exit = self
                    .gen_function_kit(body_entry, Some(final_bb), Some(cond_bb))
                    .gen_stmt(body)?
                    .exit;
                if let Some(mut body_exit) = body_exit {
                    body_exit.push_back(self.program.mem_pool.get_br(None));
                    body_exit.set_true_bb(cond_bb);
                }

                // Add condition and br to condition block
                self.exit = Some(cond_bb);
                cond_bb.set_true_bb(body_entry);
                cond_bb.set_false_bb(final_bb);
                let operand = self.gen_expr(cond)?.load(ValueType::Bool, self)?;
                let br = self.program.mem_pool.get_br(Some(operand));
                cond_bb.push_back(br);

                // Exit is final block
                self.exit = Some(final_bb);
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
                exit.set_true_bb(body_entry);
                exit.push_back(self.program.mem_pool.get_br(None));

                // Add statements and br to body block
                let body_exit = self
                    .gen_function_kit(body_entry, Some(final_bb), Some(cond_bb))
                    .gen_stmt(body)?
                    .exit;
                if let Some(mut body_exit) = body_exit {
                    body_exit.push_back(self.program.mem_pool.get_br(None));
                    body_exit.set_true_bb(cond_bb);
                }

                // Add condition and br to condition block
                self.exit = Some(cond_bb);
                cond_bb.set_true_bb(body_entry);
                cond_bb.set_false_bb(final_bb);
                let operand = self.gen_expr(cond)?.load(ValueType::Bool, self)?;
                let br = self.program.mem_pool.get_br(Some(operand));
                cond_bb.push_back(br);

                // Exit is final block
                self.exit = Some(final_bb);
            }
            Stmt::For(_, _, _, _) => {
                return Err(anyhow!("`for` statement can't be generated"))
                    .with_context(|| context!());
            }
            Stmt::Break => {
                // Add br instruction to exit block
                let br = self.program.mem_pool.get_br(None);
                exit.push_back(br);

                // When break statement appears, break_to must not be None
                let Some(break_to) = self.break_to else {
                    return Err(anyhow!("break without a valid destination"))
                        .with_context(|| context!());
                };

                // Rewrite next block to break destination
                exit.set_true_bb(break_to);

                // Exit block can't be appended further
                self.exit = None;
            }
            Stmt::Continue => {
                // Add br instruction to exit block
                let br = self.program.mem_pool.get_br(None);
                exit.push_back(br);

                // When continue statement appears, continue_to must not be None
                let Some(continue_to) = self.continue_to else {
                    return Err(anyhow!("continue without a valid destination"))
                        .with_context(|| context!());
                };

                // Rewrite next block to continue destination
                exit.set_true_bb(continue_to);

                // Exit block can't be appended further
                self.exit = None;
            }
            Stmt::Return(expr) => {
                // Add returned result to exit block
                let return_value = match expr {
                    Some(expr) => Some(self.gen_expr(expr)?.load(self.return_type.clone(), self)?),
                    None => None,
                };

                // Add ret instruction to exit block
                let ret = self.program.mem_pool.get_ret(return_value);
                exit.push_back(ret);
            }
            Stmt::Block(stmts) => {
                // Add statements to current block
                for stmt in stmts.iter() {
                    if self.exit.is_some() {
                        self.gen_stmt(stmt)?;
                    }
                }
            }
        }
        Ok(self)
    }
}