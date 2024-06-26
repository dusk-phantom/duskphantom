use crate::context;
use crate::frontend::{Expr, UnaryOp};
use crate::middle::ir::Constant;
use crate::middle::irgen::program_kit::ProgramKit;
use anyhow::{anyhow, Context};

impl<'a> ProgramKit<'a> {
    /// Generate a unary expression
    pub fn gen_const_unary(&mut self, op: &UnaryOp, expr: &Expr) -> anyhow::Result<Constant> {
        // Generate constant
        let val = self.gen_const_expr(expr)?;

        // Apply operation
        match op {
            UnaryOp::Neg => Ok(-val),
            UnaryOp::Pos => Ok(val),
            UnaryOp::Not => Ok(!val),
            _ => Err(anyhow!("unrecognized unary operator")).with_context(|| context!()),
        }
    }
}
