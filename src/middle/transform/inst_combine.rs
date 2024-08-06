use anyhow::Result;

use crate::middle::{
    ir::{instruction::InstType, Constant, InstPtr, Operand},
    Program,
};

pub fn optimize_program(program: &mut Program) -> Result<()> {
    InstCombine::new(program).run();
    Ok(())
}

#[allow(unused)]
struct InstCombine<'a> {
    program: &'a mut Program,
}

#[allow(unused, clippy::needless_return)]
impl<'a> InstCombine<'a> {
    fn new(program: &'a mut Program) -> Self {
        Self { program }
    }

    fn run(&mut self) {
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
                    self.useless_elim(inst);
                }
            }
            for bb in fun.rpo_iter() {
                for inst in bb.iter() {
                    self.combine_inst(inst);
                }
            }
            for bb in fun.rpo_iter() {
                for inst in bb.iter() {
                    self.useless_elim(inst);
                }
            }
            for bb in fun.rpo_iter() {
                for inst in bb.iter() {
                    self.make_shift(inst);
                }
            }
        }
    }

    fn useless_elim(&mut self, mut inst: InstPtr) {
        let inst_type = inst.get_type();

        // Useless instruction elimination: x + 0, x - 0, x * 1, x / 1, x >> 0, x << 0
        match inst_type {
            InstType::Add | InstType::Sub => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let Operand::Constant(constant) = rhs {
                    if constant == Constant::Int(0) {
                        inst.replace_self(&lhs);
                        return;
                    }
                }
            }
            InstType::FAdd | InstType::FSub => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let Operand::Constant(constant) = rhs {
                    if constant == Constant::Float(0.0) {
                        inst.replace_self(&lhs);
                        return;
                    }
                }
            }
            InstType::Mul | InstType::SDiv | InstType::UDiv => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let Operand::Constant(constant) = rhs {
                    if constant == Constant::Int(1) {
                        inst.replace_self(&lhs);
                        return;
                    }
                }
            }
            InstType::FMul | InstType::FDiv => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let Operand::Constant(constant) = rhs {
                    if constant == Constant::Float(1.0) {
                        inst.replace_self(&lhs);
                        return;
                    }
                }
            }
            InstType::AShr | InstType::Shl => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if let Operand::Constant(constant) = rhs {
                    if constant == Constant::Int(0) {
                        inst.replace_self(&lhs);
                        return;
                    }
                }
            }
            _ => (),
        }

        // Useless instruction elimination: x / x
        match inst_type {
            InstType::SDiv | InstType::UDiv => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if lhs == rhs {
                    inst.replace_self(&Constant::Int(1).into());
                    return;
                }
            }
            InstType::FDiv => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if lhs == rhs {
                    inst.replace_self(&Constant::Float(1.0).into());
                    return;
                }
            }
            _ => (),
        }
    }

    fn combine_inst(&mut self, mut inst: InstPtr) {
        let inst_type = inst.get_type();

        // For commutative instructions, move constant to RHS
        match inst_type {
            InstType::Add | InstType::Mul | InstType::FAdd | InstType::FMul => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if lhs.is_const() && !rhs.is_const() {
                    // Safety: swapping operand does not change use-def chain
                    unsafe {
                        let vec = inst.get_operand_mut();
                        vec.swap(0, 1);
                    }
                }
            }
            _ => (),
        }

        // Replace self add with multiplication by 2
        if inst_type == InstType::Add {
            let lhs = inst.get_operand()[0].clone();
            let rhs = inst.get_operand()[1].clone();
            if lhs == rhs {
                let new_inst = self.program.mem_pool.get_mul(lhs, Constant::Int(2).into());
                inst.insert_after(new_inst);
                inst.replace_self(&new_inst.into());
                return self.combine_inst(new_inst);
            }
        }

        // Inst combine: x + 1 - 6 -> x - 5, x * 2 * 3 -> x * 6
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
                                return self.combine_inst(new_inst);
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
                                return self.combine_inst(new_inst);
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn make_shift(&mut self, mut inst: InstPtr) {
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
                        return self.combine_inst(new_inst);
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
                        return self.combine_inst(new_inst);
                    }
                }
            }
            _ => (),
        }
    }
}
