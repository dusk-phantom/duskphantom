use crate::context;
use crate::frontend::{BinaryOp, Expr};
use crate::middle::ir::Constant;
use crate::middle::irgen::program_kit::ProgramKit;
use anyhow::{anyhow, Context};

impl<'a> ProgramKit<'a> {
    /// Generate a binary expression
    pub fn gen_const_binary(
        &mut self,
        op: &BinaryOp,
        lhs: &Expr,
        rhs: &Expr,
    ) -> anyhow::Result<Constant> {
        // Generate constants
        let lv = self.gen_const_expr(lhs)?;
        let rv = self.gen_const_expr(rhs)?;

        // Apply operation
        match op {
            BinaryOp::Add => Ok(lv + rv),
            BinaryOp::Sub => Ok(lv - rv),
            BinaryOp::Mul => Ok(lv * rv),
            BinaryOp::Div => Ok(lv / rv),
            BinaryOp::Mod => Ok(lv % rv),
            BinaryOp::Shr => Err(anyhow!("shr is not implemented yet")).with_context(|| context!()),
            BinaryOp::Shl => Err(anyhow!("shl is not implemented yet")).with_context(|| context!()),
            BinaryOp::BitAnd => Err(anyhow!("bitand not implemented")).with_context(|| context!()),
            BinaryOp::BitOr => Err(anyhow!("bitor not implemented")).with_context(|| context!()),
            BinaryOp::BitXor => Err(anyhow!("bitxor not implemented")).with_context(|| context!()),
            BinaryOp::Gt => Ok(Constant::Bool(lv > rv)),
            BinaryOp::Lt => Ok(Constant::Bool(lv < rv)),
            BinaryOp::Ge => Ok(Constant::Bool(lv >= rv)),
            BinaryOp::Le => Ok(Constant::Bool(lv <= rv)),
            BinaryOp::Eq => Ok(Constant::Bool(lv == rv)),
            BinaryOp::Ne => Ok(Constant::Bool(lv != rv)),
            BinaryOp::And => Ok(Constant::Bool(
                Into::<bool>::into(lv) && Into::<bool>::into(rv),
            )),
            BinaryOp::Or => Ok(Constant::Bool(
                Into::<bool>::into(lv) || Into::<bool>::into(rv),
            )),
        }
    }
}
