use anyhow::Result;

use crate::backend::from_self::downcast_ref;
use crate::middle::ir::instruction::misc_inst::FCmp;
use crate::middle::ir::instruction::misc_inst::FCmpOp;
use crate::middle::ir::instruction::misc_inst::ICmp;
use crate::middle::ir::instruction::misc_inst::ICmpOp;
use crate::middle::{
    ir::{instruction::InstType, Constant, InstPtr, Operand},
    Program,
};

use super::Transform;

#[allow(unused)]
pub fn optimize_program(program: &mut Program) -> Result<bool> {
    ConstantFold::new(program).run_and_log()
}

pub struct ConstantFold<'a> {
    program: &'a mut Program,
}

impl<'a> Transform for ConstantFold<'a> {
    fn get_program_mut(&mut self) -> &mut Program {
        self.program
    }

    fn name() -> String {
        "constant_fold".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        let mut changed = false;
        for func in self.program.module.functions.clone().iter() {
            if func.is_lib() {
                continue;
            }
            for bb in func.dfs_iter() {
                for inst in bb.iter() {
                    changed |= self.constant_fold_inst(inst)?;
                }
            }
        }
        Ok(changed)
    }
}

impl<'a> ConstantFold<'a> {
    pub fn new(program: &'a mut Program) -> Self {
        Self { program }
    }

    fn constant_fold_inst(&mut self, mut inst: InstPtr) -> Result<bool> {
        match inst.get_type() {
            InstType::Add | InstType::FAdd => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs + rhs;
                    inst.replace_self(&result.into());
                    return Ok(true);
                }
            }
            InstType::Sub | InstType::FSub => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs - rhs;
                    inst.replace_self(&result.into());
                    return Ok(true);
                }
            }
            InstType::Mul | InstType::FMul => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs * rhs;
                    inst.replace_self(&result.into());
                    return Ok(true);
                }
            }
            InstType::UDiv => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let lhs: u32 = lhs.into();
                    let rhs: u32 = rhs.into();
                    let result = lhs / rhs;
                    inst.replace_self(&Operand::Constant(result.into()));
                    return Ok(true);
                }
            }
            InstType::SDiv | InstType::FDiv => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs / rhs;
                    inst.replace_self(&result.into());
                    return Ok(true);
                }
            }
            InstType::URem | InstType::SRem => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs % rhs;
                    inst.replace_self(&result.into());
                    return Ok(true);
                }
            }
            InstType::Shl => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs << rhs;
                    inst.replace_self(&result.into());
                    return Ok(true);
                }
            }
            InstType::AShr => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs >> rhs;
                    inst.replace_self(&result.into());
                    return Ok(true);
                }
            }
            InstType::And => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs & rhs;
                    inst.replace_self(&result.into());
                    return Ok(true);
                }
            }
            InstType::Or => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs | rhs;
                    inst.replace_self(&result.into());
                    return Ok(true);
                }
            }
            InstType::Xor => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs ^ rhs;
                    inst.replace_self(&result.into());
                    return Ok(true);
                }
            }
            InstType::ZextTo | InstType::ItoFp | InstType::FpToI => {
                let src = inst.get_operand()[0].clone();
                if let Operand::Constant(src) = src {
                    let result = src.cast(&inst.get_value_type());
                    inst.replace_self(&result.into());
                    return Ok(true);
                }
            }
            InstType::SextTo => {
                let src = inst.get_operand()[0].clone();
                if let Operand::Constant(Constant::Bool(b)) = src {
                    let result = if b { -1 } else { 0 };
                    inst.replace_self(&Operand::Constant(result.into()));
                    return Ok(true);
                }
            }
            InstType::ICmp => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                let cmp_inst = downcast_ref::<ICmp>(inst.as_ref().as_ref());
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = match cmp_inst.op {
                        ICmpOp::Eq => lhs == rhs,
                        ICmpOp::Ne => lhs != rhs,
                        ICmpOp::Slt => lhs < rhs,
                        ICmpOp::Sle => lhs <= rhs,
                        ICmpOp::Sgt => lhs > rhs,
                        ICmpOp::Sge => lhs >= rhs,
                        ICmpOp::Ult => {
                            let lhs: u32 = lhs.into();
                            let rhs: u32 = rhs.into();
                            lhs < rhs
                        }
                        ICmpOp::Ule => {
                            let lhs: u32 = lhs.into();
                            let rhs: u32 = rhs.into();
                            lhs <= rhs
                        }
                        ICmpOp::Ugt => {
                            let lhs: u32 = lhs.into();
                            let rhs: u32 = rhs.into();
                            lhs > rhs
                        }
                        ICmpOp::Uge => {
                            let lhs: u32 = lhs.into();
                            let rhs: u32 = rhs.into();
                            lhs >= rhs
                        }
                    };
                    inst.replace_self(&Operand::Constant(result.into()));
                    return Ok(true);
                }
            }
            InstType::FCmp => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                let cmp_inst = downcast_ref::<FCmp>(inst.as_ref().as_ref());
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = match cmp_inst.op {
                        FCmpOp::False => false,
                        FCmpOp::True => true,
                        FCmpOp::Oeq => lhs == rhs,
                        FCmpOp::One => lhs != rhs,
                        FCmpOp::Olt => lhs < rhs,
                        FCmpOp::Ole => lhs <= rhs,
                        FCmpOp::Ogt => lhs > rhs,
                        FCmpOp::Oge => lhs >= rhs,
                        FCmpOp::Ueq => {
                            let lhs: f32 = lhs.into();
                            let rhs: f32 = rhs.into();
                            lhs == rhs || (lhs.is_nan() && rhs.is_nan())
                        }
                        FCmpOp::Une => {
                            let lhs: f32 = lhs.into();
                            let rhs: f32 = rhs.into();
                            lhs.is_nan() || rhs.is_nan() || lhs != rhs
                        }
                        FCmpOp::Ult => {
                            let lhs: f32 = lhs.into();
                            let rhs: f32 = rhs.into();
                            lhs < rhs || (lhs.is_nan() && !rhs.is_nan())
                        }
                        FCmpOp::Ule => {
                            let lhs: f32 = lhs.into();
                            let rhs: f32 = rhs.into();
                            lhs <= rhs || (lhs.is_nan() && !rhs.is_nan())
                        }
                        FCmpOp::Ugt => {
                            let lhs: f32 = lhs.into();
                            let rhs: f32 = rhs.into();
                            lhs > rhs || (!lhs.is_nan() && rhs.is_nan())
                        }
                        FCmpOp::Uge => {
                            let lhs: f32 = lhs.into();
                            let rhs: f32 = rhs.into();
                            lhs >= rhs || (!lhs.is_nan() && rhs.is_nan())
                        }
                        _ => todo!(),
                    };
                    inst.replace_self(&Operand::Constant(result.into()));
                    return Ok(true);
                }
            }
            _ => (),
        }
        Ok(false)
    }
}
