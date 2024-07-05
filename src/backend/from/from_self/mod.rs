pub use anyhow::Result;

mod builder;
mod instruction;
mod operand;

pub use builder::*;
#[allow(unused)]
pub use instruction::*;
#[allow(unused)]
pub use operand::*;

use super::super::prog;
use crate::errors::*;
use crate::middle;

/// 中端层面，地址是唯一的
/// 因此我可以将地址作为 id
/// 用在 parameter 和 instruction 上
type Address = usize;

#[allow(unused)]
pub fn gen(program: &middle::Program) -> Result<prog::Program, BackendError> {
    // TODO
    Ok(prog::Program {
        entry: None,
        modules: vec![],
    })
}

#[macro_export]
macro_rules! ssa2tac_binary_usual {
    // ssa2tac_binary_usual!(middle::ir::instruction::binary_inst::Xor, Xor, XorInst)
    ($inst:ident, $regs:ident, $reg_gener:ident, $ssa_ty:ident, $tac_enum:ident, $tac_inst:ident) => {{
        let dinst =
            downcast_ref::<middle::ir::instruction::binary_inst::$ssa_ty>($inst.as_ref().as_ref());
        let lhs = Self::local_operand_from(dinst.get_lhs(), $regs).with_context(|| context!())?;
        let rhs = Self::local_operand_from(dinst.get_rhs(), $regs).with_context(|| context!())?;
        let dst = $reg_gener.gen_virtual_usual_reg();
        $regs.insert(dinst as *const _ as Address, dst);
        let inst = $tac_inst::new(dst.into(), lhs, rhs);
        Ok(vec![Inst::$tac_enum(inst)])
    }};
}

#[macro_export]
macro_rules! ssa2tac_binary_float {
    // ssa2tac_binary_usual!(middle::ir::instruction::binary_inst::Xor, Xor, XorInst)
    ($inst:ident, $regs:ident, $reg_gener:ident, $ssa_ty:ident, $tac_enum:ident, $tac_inst:ident) => {{
        let dinst =
            downcast_ref::<middle::ir::instruction::binary_inst::$ssa_ty>($inst.as_ref().as_ref());
        let lhs = Self::local_operand_from(dinst.get_lhs(), $regs).with_context(|| context!())?;
        let rhs = Self::local_operand_from(dinst.get_rhs(), $regs).with_context(|| context!())?;
        let dst = $reg_gener.gen_virtual_float_reg();
        $regs.insert(dinst as *const _ as Address, dst);
        let inst = $tac_inst::new(dst.into(), lhs, rhs);
        Ok(vec![Inst::$tac_enum(inst)])
    }};
}
