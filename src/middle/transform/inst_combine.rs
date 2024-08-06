use crate::middle::{
    ir::{instruction::InstType, Constant, InstPtr, Operand},
    Program,
};

struct InstCombine<'a> {
    program: &'a mut Program,
}

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
                    self.combine_inst(inst);
                }
            }
        }
    }

    fn canonicalize_inst(&mut self, mut inst: InstPtr) {
        // For commutative instructions, move constant to RHS
        match inst.get_type() {
            InstType::Add | InstType::Mul | InstType::FAdd | InstType::FMul => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if lhs.is_const() && !rhs.is_const() {
                    inst.set_operand(0, rhs);
                    inst.set_operand(1, lhs);
                }
            }
            _ => (),
        }

        // For self add, replace with shift
        match inst.get_type() {
            InstType::Add => {
                let lhs = inst.get_operand()[0].clone();
                let rhs = inst.get_operand()[1].clone();
                if lhs == rhs {
                    let shl = self
                        .program
                        .mem_pool
                        .get_shl(lhs.into(), Operand::Constant(1.into()));
                    inst.replace_self(&shl.into());
                }
            }
            _ => (),
        }

        // Useless instruction elimination
        match inst.get_type() {
            InstType::Add | InstType::Sub => {
                let lhs = &inst.get_operand()[0];
                let rhs = &inst.get_operand()[1];
                if rhs.is_const() && rhs.get_const().unwrap() == Constant::Int(0) {
                    inst.clone().replace_self(lhs);
                }
            }
            _ => (),
        }
    }

    fn combine_inst(&mut self, inst: InstPtr) {}
}
