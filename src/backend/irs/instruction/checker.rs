use var::Var;

use super::*;
pub trait IRChecker: ProgramChecker {}

pub trait ProgramChecker: ModuleChecker {
    fn check_valid(&self, program: &Program) -> bool;
}

pub trait ModuleChecker: VarChecker + FuncChecker {
    fn check_valid(&self, module: &Module) -> bool;
}

pub trait VarChecker {
    fn check_valid(&self, var: &Var) -> bool;
}

pub trait FuncChecker: BBChecker {
    fn check_valid(&self, func: &Func) -> bool;
}

pub trait BBChecker {
    fn check_valid(&self, bb: &Block) -> bool;
}
pub trait InstChecker {
    fn check_valid(&self, inst: &Inst) -> bool;
}

pub struct Riscv;

impl IRChecker for Riscv {}
impl ProgramChecker for Riscv {
    fn check_valid(&self, program: &Program) -> bool {
        for module in program.modules.iter() {
            if !ModuleChecker::check_valid(self, module) {
                return false;
            }
        }
        true
    }
}
impl ModuleChecker for Riscv {
    fn check_valid(&self, module: &Module) -> bool {
        for var in module.global.iter() {
            if !VarChecker::check_valid(self, var) {
                return false;
            }
        }
        for func in module.funcs.iter() {
            if !FuncChecker::check_valid(self, func) {
                return false;
            }
        }
        true
    }
}
impl VarChecker for Riscv {
    #[allow(unused_variables)]
    fn check_valid(&self, var: &Var) -> bool {
        true
    }
}
impl FuncChecker for Riscv {
    fn check_valid(&self, func: &Func) -> bool {
        for bb in func.iter_bbs() {
            if !BBChecker::check_valid(self, bb) {
                return false;
            }
        }
        true
    }
}

impl BBChecker for Riscv {
    fn check_valid(&self, bb: &Block) -> bool {
        for inst in bb.insts() {
            if !InstChecker::check_valid(self, inst) {
                return false;
            }
        }
        true
    }
}
impl InstChecker for Riscv {
    fn check_valid(&self, inst: &Inst) -> bool {
        match inst {
            Inst::Add(add) => self.check_add(add),
            Inst::Sub(sub) => self.check_sub(sub),
            Inst::Mul(mul) => self.check_mul(mul),
            Inst::Rem(rem) => self.check_rem(rem),
            Inst::Div(div) => self.check_div(div),
            Inst::Sll(_) => true,
            Inst::Srl(_) => true,
            Inst::SRA(_) => true,
            Inst::And(_) => true,
            Inst::Not(not) => self.check_not(not),
            Inst::Or(_) => true,
            Inst::Xor(_) => true,
            Inst::Slt(_) => true,
            Inst::Snez(snez) => self.check_snez(snez),
            Inst::Seqz(seqz) => self.check_seqz(seqz),
            Inst::Neg(_) => true,
            Inst::Mv(_) => true,
            Inst::Ld(ld) => self.check_ld(ld),
            Inst::Sd(sd) => self.check_sd(sd),
            Inst::Sw(sw) => self.check_sw(sw),
            Inst::Lw(lw) => self.check_lw(lw),
            Inst::Lla(_) => true,
            // special inst to temporary express the load and store operation ,should not to keep in the final ir
            Inst::Load(_) => false,
            Inst::Store(_) => false,
            // special inst to temporary express the local address ,should not to keep in the final ir
            Inst::LocalAddr(_) => false,

            Inst::Jmp(_) => true,
            Inst::Beq(_) => true,
            Inst::Bne(_) => true,
            Inst::Bge(_) => true,
            Inst::Blt(_) => true,
            Inst::Bgt(_) => true,
            Inst::Ble(_) => true,
            Inst::Call(_) => true,
            Inst::Tail(_) => true,
            Inst::Li(li) => self.check_li(li),
            Inst::F2i(f2i) => self.check_f2i(f2i),
            Inst::I2f(i2f) => self.check_i2f(i2f),
            Inst::Ret => true,
        }
    }
}

impl Riscv {
    fn check_add(&self, add: &AddInst) -> bool {
        matches!(add.dst(), Operand::Reg(_))
            && matches!(add.lhs(), Operand::Reg(_))
            && (matches!(add.rhs(), Operand::Reg(_))
                || Self::check_imm_op_in_i_type_inst(add.rhs()))
    }

    fn check_snez(&self, snez: &SnezInst) -> bool {
        matches!(snez.dst(), Operand::Reg(_)) && matches!(snez.src(), Operand::Reg(_))
    }

    fn check_seqz(&self, seqz: &SeqzInst) -> bool {
        matches!(seqz.dst(), Operand::Reg(_)) && matches!(seqz.src(), Operand::Reg(_))
    }

    fn check_not(&self, not: &NotInst) -> bool {
        matches!(not.dst(), Operand::Reg(_)) && matches!(not.src(), Operand::Reg(_))
    }

    fn check_f2i(&self, f2i: &F2iInst) -> bool {
        (match f2i.dst() {
            Operand::Reg(r) => r.is_usual(),
            _ => false,
        }) && (match f2i.src() {
            Operand::Reg(r) => r.is_float(),
            _ => false,
        })
    }

    fn check_i2f(&self, i2f: &I2fInst) -> bool {
        (match i2f.dst() {
            Operand::Reg(r) => r.is_float(),
            _ => false,
        }) && (match i2f.src() {
            Operand::Reg(r) => r.is_usual(),
            _ => false,
        })
    }

    fn check_li(&self, li: &LiInst) -> bool {
        matches!(li.dst(), Operand::Reg(_)) && matches!(li.src(), Operand::Imm(_))
    }

    fn check_rem(&self, rem: &RemInst) -> bool {
        matches!(rem.dst(), Operand::Reg(_))
            && matches!(rem.lhs(), Operand::Reg(_))
            && matches!(rem.rhs(), Operand::Reg(_))
    }

    fn check_div(&self, div: &DivInst) -> bool {
        matches!(div.dst(), Operand::Reg(_))
            && matches!(div.lhs(), Operand::Reg(_))
            && matches!(div.rhs(), Operand::Reg(_))
    }

    fn check_mul(&self, mul: &MulInst) -> bool {
        matches!(mul.dst(), Operand::Reg(_))
            && matches!(mul.lhs(), Operand::Reg(_))
            && (matches!(mul.rhs(), Operand::Reg(_))
                || Self::check_imm_op_in_i_type_inst(mul.rhs()))
    }

    fn check_sub(&self, sub: &SubInst) -> bool {
        matches!(sub.dst(), Operand::Reg(_))
            && matches!(sub.lhs(), Operand::Reg(_))
            && matches!(sub.rhs(), Operand::Reg(_))
    }
    fn check_ld(&self, ld: &LdInst) -> bool {
        Self::check_imm_in_i_type_inst(ld.offset())
    }
    fn check_sd(&self, sd: &SdInst) -> bool {
        Self::check_imm_in_i_type_inst(sd.offset())
    }
    fn check_lw(&self, lw: &LwInst) -> bool {
        Self::check_imm_in_i_type_inst(lw.offset())
    }
    fn check_sw(&self, sw: &SwInst) -> bool {
        Self::check_imm_in_i_type_inst(sw.offset())
    }
    fn check_imm_in_i_type_inst(imm: &Imm) -> bool {
        imm.in_limit(12)
    }
    fn check_imm_op_in_i_type_inst(op: &Operand) -> bool {
        if let Operand::Imm(imm) = op {
            imm.in_limit(12)
        } else {
            false
        }
    }
}
