use anyhow::Result;
use std::cmp;
use std::ops;

use crate::backend::from_self::downcast_ref;
use crate::middle::ir::instruction::misc_inst::FCmp;
use crate::middle::ir::instruction::misc_inst::FCmpOp;
use crate::middle::ir::instruction::misc_inst::ICmp;
use crate::middle::ir::instruction::misc_inst::ICmpOp;
use crate::middle::ir::ValueType;
use crate::middle::{
    ir::{instruction::InstType, BBPtr, Constant, FunPtr, InstPtr, Operand},
    Program,
};

#[allow(unused)]
pub fn optimize_program(program: &mut Program) -> Result<()> {
    ConstantFold::new(program).constant_fold();
    Ok(())
}

pub struct ConstantFold<'a> {
    program: &'a mut Program,
}

impl<'a> ConstantFold<'a> {
    pub fn new(program: &'a mut Program) -> Self {
        Self { program }
    }

    pub fn constant_fold(&mut self) {
        self.program
            .module
            .functions
            .clone()
            .iter()
            .filter(|f| !f.is_lib())
            .for_each(|func| self.constant_fold_func(func));
    }

    pub fn constant_fold_func(&mut self, func: &FunPtr) {
        func.rpo_iter().for_each(|bb| self.constant_fold_block(bb));
    }

    pub fn constant_fold_block(&mut self, bb: BBPtr) {
        bb.iter().for_each(|inst| self.constant_fold_inst(inst));
    }

    pub fn constant_fold_inst(&mut self, mut inst: InstPtr) {
        match inst.get_type() {
            InstType::Add | InstType::FAdd => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs + rhs;
                    inst.replace_self(&result.into());
                }
            }
            InstType::Sub | InstType::FSub => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs - rhs;
                    inst.replace_self(&result.into());
                }
            }
            InstType::Mul | InstType::FMul => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs * rhs;
                    inst.replace_self(&result.into());
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
                }
            }
            InstType::SDiv | InstType::FDiv => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs / rhs;
                    inst.replace_self(&result.into());
                }
            }
            InstType::URem | InstType::SRem => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs % rhs;
                    inst.replace_self(&result.into());
                }
            }
            InstType::Shl => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs << rhs;
                    inst.replace_self(&result.into());
                }
            }
            InstType::LShr => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let lhs: u32 = lhs.into();
                    let rhs: u32 = rhs.into();
                    let result = lhs >> rhs;
                    inst.replace_self(&Operand::Constant(result.into()));
                }
            }
            InstType::AShr => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs >> rhs;
                    inst.replace_self(&result.into());
                }
            }
            InstType::And => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs & rhs;
                    inst.replace_self(&result.into());
                }
            }
            InstType::Or => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs | rhs;
                    inst.replace_self(&result.into());
                }
            }
            InstType::Xor => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let (Operand::Constant(lhs), Operand::Constant(rhs)) = (lhs, rhs) {
                    let result = lhs ^ rhs;
                    inst.replace_self(&result.into());
                }
            }
            InstType::ZextTo | InstType::ItoFp | InstType::FpToI => {
                let src = inst.get_operand()[0].clone();
                if let Operand::Constant(src) = src {
                    let result = src.cast(&inst.get_value_type());
                    inst.replace_self(&result.into());
                }
            }
            InstType::SextTo => {
                let src = inst.get_operand()[0].clone();
                if let Operand::Constant(Constant::Bool(b)) = src {
                    let result = if b { -1 } else { 0 };
                    inst.replace_self(&Operand::Constant(result.into()));
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
                }
            }
            _ => (),
        }
    }
}

impl Constant {
    pub fn cast(self, ty: &ValueType) -> Self {
        match ty {
            ValueType::Int => Into::<i32>::into(self).into(),
            ValueType::Float => Into::<f32>::into(self).into(),
            ValueType::Bool => Into::<bool>::into(self).into(),
            ValueType::Array(element_ty, _) => {
                let arr = match self {
                    Constant::Array(arr) => arr,
                    _ => panic!("Cannot convert {} to array", self),
                };
                Constant::Array(arr.into_iter().map(|x| x.cast(element_ty)).collect())
            }
            _ => self,
        }
    }
}

/// Override operators for constant
impl ops::Neg for Constant {
    type Output = Constant;

    fn neg(self) -> Self::Output {
        let ty = self.get_type();
        match ty {
            ValueType::Float => (-Into::<f32>::into(self)).into(),
            ValueType::Int | ValueType::Bool => (-Into::<i32>::into(self)).into(),
            _ => todo!(),
        }
    }
}

impl ops::Not for Constant {
    type Output = Constant;

    fn not(self) -> Self::Output {
        (!Into::<bool>::into(self)).into()
    }
}

impl ops::Add for Constant {
    type Output = Constant;

    fn add(self, rhs: Constant) -> Self::Output {
        let ty = self.get_type();
        match ty {
            ValueType::Float => (Into::<f32>::into(self) + Into::<f32>::into(rhs)).into(),
            ValueType::Int | ValueType::Bool => {
                (Into::<i32>::into(self) + Into::<i32>::into(rhs)).into()
            }
            _ => todo!(),
        }
    }
}

impl ops::Sub for Constant {
    type Output = Constant;

    fn sub(self, rhs: Constant) -> Self::Output {
        let ty = self.get_type();
        match ty {
            ValueType::Float => (Into::<f32>::into(self) - Into::<f32>::into(rhs)).into(),
            ValueType::Int | ValueType::Bool => {
                (Into::<i32>::into(self) - Into::<i32>::into(rhs)).into()
            }
            _ => todo!(),
        }
    }
}

impl ops::Mul for Constant {
    type Output = Constant;

    fn mul(self, rhs: Constant) -> Self::Output {
        let ty = self.get_type();
        match ty {
            ValueType::Float => (Into::<f32>::into(self) * Into::<f32>::into(rhs)).into(),
            ValueType::Int | ValueType::Bool => {
                (Into::<i32>::into(self) * Into::<i32>::into(rhs)).into()
            }
            _ => todo!(),
        }
    }
}

impl ops::Div for Constant {
    type Output = Constant;

    fn div(self, rhs: Constant) -> Self::Output {
        let ty = self.get_type();
        match ty {
            ValueType::Float => (Into::<f32>::into(self) / Into::<f32>::into(rhs)).into(),
            ValueType::Int | ValueType::Bool => {
                (Into::<i32>::into(self) / Into::<i32>::into(rhs)).into()
            }
            _ => todo!(),
        }
    }
}

impl ops::Rem for Constant {
    type Output = Constant;

    fn rem(self, rhs: Constant) -> Self::Output {
        (Into::<i32>::into(self) % Into::<i32>::into(rhs)).into()
    }
}

impl ops::Shl for Constant {
    type Output = Constant;

    fn shl(self, rhs: Constant) -> Self::Output {
        (Into::<i32>::into(self) << Into::<i32>::into(rhs)).into()
    }
}

impl ops::Shr for Constant {
    type Output = Constant;

    fn shr(self, rhs: Constant) -> Self::Output {
        (Into::<i32>::into(self) >> Into::<i32>::into(rhs)).into()
    }
}

impl ops::BitAnd for Constant {
    type Output = Constant;

    fn bitand(self, rhs: Constant) -> Self::Output {
        (Into::<i32>::into(self) & Into::<i32>::into(rhs)).into()
    }
}

impl ops::BitOr for Constant {
    type Output = Constant;

    fn bitor(self, rhs: Constant) -> Self::Output {
        (Into::<i32>::into(self) | Into::<i32>::into(rhs)).into()
    }
}

impl ops::BitXor for Constant {
    type Output = Constant;

    fn bitxor(self, rhs: Constant) -> Self::Output {
        (Into::<i32>::into(self) ^ Into::<i32>::into(rhs)).into()
    }
}

impl cmp::PartialOrd for Constant {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let ty = self.get_type();
        match ty {
            ValueType::Float => {
                Into::<f32>::into(self.clone()).partial_cmp(&Into::<f32>::into(other.clone()))
            }
            ValueType::Int => {
                Into::<i32>::into(self.clone()).partial_cmp(&Into::<i32>::into(other.clone()))
            }
            ValueType::Bool => {
                Into::<bool>::into(self.clone()).partial_cmp(&Into::<bool>::into(other.clone()))
            }
            _ => todo!(),
        }
    }
}
