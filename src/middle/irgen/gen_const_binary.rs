use crate::context;
use crate::frontend::{BinaryOp, Expr};
use crate::middle::ir::Constant;
use crate::middle::irgen::program_kit::ProgramKit;
use anyhow::{anyhow, Context};

impl<'a> ProgramKit<'a> {
    /// Generate a binary expression
    pub fn gen_const_binary(
        &self,
        head: &Expr,
        tail: &[(BinaryOp, Expr)],
    ) -> anyhow::Result<Constant> {
        let mut head = self.gen_const_expr(head)?;

        // Iterate through the tail
        for (op, expr) in tail {
            let expr = self.gen_const_expr(expr)?;
            head = match op {
                BinaryOp::Add => head + expr,
                BinaryOp::Sub => head - expr,
                BinaryOp::Mul => head * expr,
                BinaryOp::Div => head / expr,
                BinaryOp::Mod => head % expr,
                BinaryOp::Shr => {
                    return Err(anyhow!("shr is not implemented yet")).with_context(|| context!());
                }
                BinaryOp::Shl => {
                    return Err(anyhow!("shl is not implemented yet")).with_context(|| context!());
                }
                BinaryOp::BitAnd => {
                    return Err(anyhow!("bitand not implemented")).with_context(|| context!());
                }
                BinaryOp::BitOr => {
                    return Err(anyhow!("bitor not implemented")).with_context(|| context!());
                }
                BinaryOp::BitXor => {
                    return Err(anyhow!("bitxor not implemented")).with_context(|| context!());
                }
                BinaryOp::Gt => Constant::Bool(head > expr),
                BinaryOp::Lt => Constant::Bool(head < expr),
                BinaryOp::Ge => Constant::Bool(head >= expr),
                BinaryOp::Le => Constant::Bool(head <= expr),
                BinaryOp::Eq => Constant::Bool(head == expr),
                BinaryOp::Ne => Constant::Bool(head != expr),
                BinaryOp::And => {
                    Constant::Bool(Into::<bool>::into(head) && Into::<bool>::into(expr))
                }
                BinaryOp::Or => {
                    Constant::Bool(Into::<bool>::into(head) || Into::<bool>::into(expr))
                }
            };
        }
        Ok(head)
    }
}
