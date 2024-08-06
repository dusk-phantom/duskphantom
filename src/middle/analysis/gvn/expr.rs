use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use crate::middle::ir::{InstPtr, Operand};

use super::context::Context;

#[allow(unused)]
pub enum Expr<'a> {
    Assoc(InstPtr, HashMap<Box<Expr<'a>>, i32>, Operand),
    Inst(&'a Context<'a>, InstPtr),
    Operand(Operand),
}

impl Hash for Expr<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Expr::Assoc(_, content, op) => {
                // Hash map is not mutable, so iterate order is fixed
                // Hash each expression and its quantity
                content.iter().for_each(|(expr, i)| {
                    expr.hash(state);
                    i.hash(state);
                });

                // Hash constant part
                op.hash(state);
            }
            Expr::Inst(ctx, inst) => {
                // Hash instruction type
                inst.get_type().hash(state);

                // Hash each operand
                inst.get_operand()
                    .iter()
                    .for_each(|op| ctx.get_expr(op).as_ref().hash(state));
            }
            Expr::Operand(op) => op.hash(state),
        }
    }
}
