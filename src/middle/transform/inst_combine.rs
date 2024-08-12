use anyhow::Result;

use crate::{
    backend::from_self::downcast_ref,
    middle::{
        ir::{
            instruction::{
                memory_op_inst::GetElementPtr,
                misc_inst::{FCmp, FCmpOp, ICmp, ICmpOp},
                InstType,
            },
            Constant, InstPtr, Operand,
        },
        Program,
    },
};

use super::Transform;

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    InstCombine::new(program).run_and_log()
}

pub struct InstCombine<'a> {
    program: &'a mut Program,
}

impl<'a> Transform for InstCombine<'a> {
    fn name() -> String {
        "inst_combine".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        let mut changed = false;
        for fun in self
            .program
            .module
            .functions
            .clone()
            .iter()
            .filter(|f| !f.is_lib())
        {
            for bb in fun.rpo_iter() {
                for inst in bb.iter() {
                    changed |= self.symbolic_eval(inst)?;
                }
            }
            for bb in fun.rpo_iter() {
                for inst in bb.iter() {
                    changed |= self.make_shift(inst)?;
                }
            }
        }
        Ok(changed)
    }
}

impl<'a> InstCombine<'a> {
    pub fn new(program: &'a mut Program) -> Self {
        Self { program }
    }

    fn symbolic_eval(&mut self, mut inst: InstPtr) -> Result<bool> {
        let inst_type = inst.get_type();

        // For commutative instructions, move constant to RHS
        let mut changed = false;
        match inst_type {
            InstType::Add | InstType::Mul | InstType::FAdd | InstType::FMul => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if lhs.is_const() && !rhs.is_const() {
                    // Safety: swapping operand does not change use-def chain
                    unsafe {
                        let vec = inst.get_operand_mut();
                        vec.swap(0, 1);
                        changed = true;
                    }
                }
            }
            _ => (),
        }

        // Constant folding
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

        // Useless instruction elimination:
        // x + 0, x - 0, x * 1, x / 1, x >> 0, x << 0,
        // 0 / x, x * 0, phi x, ..., x, br true, ...
        match inst_type {
            InstType::Add | InstType::Sub => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let Operand::Constant(constant) = rhs {
                    if constant == Constant::Int(0) {
                        inst.replace_self(&lhs);
                        return Ok(true);
                    }
                }
            }
            InstType::FAdd | InstType::FSub => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let Operand::Constant(constant) = rhs {
                    if constant == Constant::Float(0.0) {
                        inst.replace_self(&lhs);
                        return Ok(true);
                    }
                }
            }
            InstType::Mul | InstType::SDiv | InstType::UDiv => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let Operand::Constant(rhs) = rhs {
                    if rhs == Constant::Int(1) {
                        inst.replace_self(&lhs);
                        return Ok(true);
                    } else if rhs == Constant::Int(0) {
                        inst.replace_self(&rhs.into());
                        return Ok(true);
                    }
                }
            }
            InstType::FMul | InstType::FDiv => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let Operand::Constant(rhs) = rhs {
                    if rhs == Constant::Float(1.0) {
                        inst.replace_self(&lhs);
                        return Ok(true);
                    }
                }
                if let Operand::Constant(lhs) = lhs {
                    if lhs == Constant::Float(0.0) {
                        inst.replace_self(&lhs.into());
                        return Ok(true);
                    }
                }
            }
            InstType::AShr | InstType::Shl => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let Operand::Constant(rhs) = rhs {
                    if rhs == Constant::Int(0) {
                        inst.replace_self(&lhs);
                        return Ok(true);
                    }
                }
                if let Operand::Constant(lhs) = lhs {
                    if lhs == Constant::Int(0) {
                        inst.replace_self(&lhs.into());
                        return Ok(true);
                    }
                }
            }
            InstType::Phi => {
                let first = inst.get_operand()[0].clone();
                let all_same = inst.get_operand().iter().all(|op| *op == first);
                if all_same {
                    inst.replace_self(&first);
                    return Ok(true);
                }
            }
            _ => (),
        }

        // Useless instruction elimination: x / x, x - x, x + x (to x * 2)
        match inst_type {
            InstType::SDiv | InstType::UDiv => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if lhs == rhs {
                    inst.replace_self(&Constant::Int(1).into());
                    return Ok(true);
                }
            }
            InstType::FDiv => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if lhs == rhs {
                    inst.replace_self(&Constant::Float(1.0).into());
                    return Ok(true);
                }
            }
            InstType::Sub => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if lhs == rhs {
                    inst.replace_self(&Constant::Int(0).into());
                    return Ok(true);
                }
            }
            InstType::Add => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if lhs == rhs {
                    let new_inst = self.program.mem_pool.get_mul(lhs, Constant::Int(2).into());
                    inst.insert_after(new_inst);
                    inst.replace_self(&new_inst.into());
                    return Ok(true);
                }
            }
            _ => (),
        }

        // Merge GEP instruction
        if inst_type == InstType::GetElementPtr {
            let ptr = inst.get_operand()[0].clone();
            if let Operand::Instruction(ptr) = ptr {
                if ptr.get_type() == InstType::GetElementPtr {
                    // Outer GEP: getelementptr ty1, inner, i1, ..., in
                    // Inner GEP: getelementptr ty2, alloc, j1, ..., jm
                    // Merged GEP: getelementptr ty2, alloc, j1, ..., jm + i1, ..., in
                    let m = ptr.get_operand().len() - 1;

                    // Create instruction for jm + i1
                    let add = self
                        .program
                        .mem_pool
                        .get_add(ptr.get_operand()[m].clone(), inst.get_operand()[1].clone());
                    inst.insert_before(add);

                    // Create a list of all operands
                    let operands = [
                        ptr.get_operand()[1..m].to_vec(),
                        vec![add.into()],
                        inst.get_operand()[2..].to_vec(),
                    ]
                    .concat();

                    // Create new GEP instruction
                    let gep = downcast_ref::<GetElementPtr>(ptr.as_ref().as_ref());
                    let new_inst = self.program.mem_pool.get_getelementptr(
                        gep.element_type.clone(),
                        ptr.get_operand()[0].clone(),
                        operands,
                    );

                    // Replace outer GEP with new GEP
                    inst.insert_after(new_inst);
                    inst.replace_self(&new_inst.into());
                    return Ok(true);
                }
            }
        }

        // Inst combine: (x * n) + x = x * (n + 1), (x * n) - x = x * (n - 1)
        match inst_type {
            InstType::Add | InstType::Sub => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();

                // Check if "lhs is mul", "rhs is same as lhs_lhs" and "lhs_rhs is int constant"
                if let Operand::Instruction(lhs) = lhs {
                    if lhs.get_type() == InstType::Mul {
                        let lhs_lhs = lhs.get_operand()[0].clone();
                        let lhs_rhs = lhs.get_operand()[1].clone();

                        if lhs_lhs == rhs {
                            if let Operand::Constant(Constant::Int(lhs_rhs)) = lhs_rhs {
                                let new_rhs = if inst_type == InstType::Add {
                                    lhs_rhs + 1
                                } else {
                                    lhs_rhs - 1
                                };
                                let new_inst = self
                                    .program
                                    .mem_pool
                                    .get_mul(lhs_lhs, Constant::Int(new_rhs).into());
                                inst.insert_after(new_inst);
                                inst.replace_self(&new_inst.into());
                                return Ok(true);
                            }
                        }
                    }
                }
            }
            _ => (),
        }

        // Inst combine: x + 1 - 6 -> x - 5, x * 2 * 3 -> x * 6, x / 2 / 3 -> x / 6
        match inst_type {
            InstType::Add | InstType::Sub => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();

                // Check if "rhs is constant" and "lhs is add or sub"
                if let Operand::Constant(rhs) = rhs {
                    if let Operand::Instruction(lhs) = lhs {
                        let lhs_type = lhs.get_type();
                        if matches!(lhs_type, InstType::Add | InstType::Sub) {
                            let lhs_lhs = lhs.get_operand()[0].clone();
                            let lhs_rhs = lhs.get_operand()[1].clone();

                            // Combine inst if "lhs_rhs is constant"
                            if let Operand::Constant(lhs_rhs) = lhs_rhs {
                                let new_rhs = lhs_rhs.apply(lhs_type) + rhs.apply(inst_type);
                                let new_inst =
                                    self.program.mem_pool.get_add(lhs_lhs, new_rhs.into());
                                inst.insert_after(new_inst);
                                inst.replace_self(&new_inst.into());
                                return Ok(true);
                            }
                        }
                    }
                }
            }
            InstType::Mul => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();

                // Check if "rhs is constant" and "lhs is mul"
                if let Operand::Constant(rhs) = rhs {
                    if let Operand::Instruction(lhs) = lhs {
                        if lhs.get_type() == InstType::Mul {
                            let lhs_lhs = lhs.get_operand()[0].clone();
                            let lhs_rhs = lhs.get_operand()[1].clone();

                            // Combine inst if "lhs_rhs is constant"
                            if let Operand::Constant(lhs_rhs) = lhs_rhs {
                                let new_rhs = lhs_rhs * rhs;
                                let new_inst =
                                    self.program.mem_pool.get_mul(lhs_lhs, new_rhs.into());
                                inst.insert_after(new_inst);
                                inst.replace_self(&new_inst.into());
                                return Ok(true);
                            }
                        }
                    }
                }
            }
            InstType::SDiv => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();

                // Check if "rhs is constant" and "lhs is div"
                if let Operand::Constant(Constant::Int(rhs)) = rhs {
                    if let Operand::Instruction(lhs) = lhs {
                        if lhs.get_type() == InstType::SDiv {
                            let lhs_lhs = lhs.get_operand()[0].clone();
                            let lhs_rhs = lhs.get_operand()[1].clone();

                            // Combine inst if "lhs_rhs is constant"
                            if let Operand::Constant(Constant::Int(lhs_rhs)) = lhs_rhs {
                                let (new_rhs, overflow) = lhs_rhs.overflowing_mul(rhs);

                                // If overflow, instruction result is zero
                                if overflow {
                                    inst.replace_self(&Constant::Int(0).into());
                                    return Ok(true);
                                }

                                // Otherwise, combine division factors
                                let new_inst = self
                                    .program
                                    .mem_pool
                                    .get_sdiv(lhs_lhs, Constant::Int(new_rhs).into());
                                inst.insert_after(new_inst);
                                inst.replace_self(&new_inst.into());
                                return Ok(true);
                            }
                        }
                    }
                }
            }
            _ => (),
        }
        Ok(changed)
    }

    fn make_shift(&mut self, mut inst: InstPtr) -> Result<bool> {
        let inst_type = inst.get_type();

        // Replace mul or div with power of 2 with shift
        match inst_type {
            InstType::Mul => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let Operand::Constant(Constant::Int(rhs)) = rhs {
                    if rhs.count_ones() == 1 {
                        let new_inst = self
                            .program
                            .mem_pool
                            .get_shl(lhs, Operand::Constant(rhs.trailing_zeros().into()));
                        inst.insert_after(new_inst);
                        inst.replace_self(&new_inst.into());
                        return Ok(true);
                    }
                }
            }
            InstType::SDiv => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let Operand::Constant(Constant::Int(rhs)) = rhs {
                    if rhs.count_ones() == 1 {
                        let new_inst = self
                            .program
                            .mem_pool
                            .get_ashr(lhs, Operand::Constant(rhs.trailing_zeros().into()));
                        inst.insert_after(new_inst);
                        inst.replace_self(&new_inst.into());
                        return Ok(true);
                    }
                }
            }
            _ => (),
        }
        Ok(false)
    }
}
