use std::collections::HashMap;

use crate::{
    middle::ir::{InstPtr, Operand},
    utils::maybe_owned::MaybeOwned,
};

use super::expr::Expr;

pub struct Context<'a> {
    exprs: HashMap<InstPtr, Expr<'a>>,
}

impl Context<'_> {
    /// Get expression of an instruction.
    /// Value number for the instruction is retrieved by hashing the expression.
    pub fn get_expr(&self, op: &Operand) -> MaybeOwned<Expr> {
        // If operand is global variable / parameter / constant, number by itself
        let Operand::Instruction(inst) = op else {
            return Expr::Operand(op.clone()).into();
        };

        // If instruction is not yet numbered, it is defined via back edge
        // Assign a temporary number by instruction pointer
        let Some(expr) = self.exprs.get(inst) else {
            return Expr::Operand(op.clone()).into();
        };

        // Instruction is numbered, return the expression carrying the number
        expr.into()
    }
}
