// Copyright 2024 Duskphantom Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

pub struct Riscv;

impl IRChecker for Riscv {}
impl ProgramChecker for Riscv {
    fn check_prog(&self, program: &Program) -> bool {
        for module in program.modules.iter() {
            if !ModuleChecker::check_mdl(self, module) {
                return false;
            }
        }
        true
    }
}
impl ModuleChecker for Riscv {
    fn check_mdl(&self, module: &Module) -> bool {
        for var in module.global.iter() {
            if !VarChecker::check_var(self, var) {
                return false;
            }
        }
        for func in module.funcs.iter() {
            if !FuncChecker::check_func(self, func) {
                return false;
            }
        }
        true
    }
}
impl VarChecker for Riscv {
    #[allow(unused_variables)]
    fn check_var(&self, var: &Var) -> bool {
        true
    }
}
impl FuncChecker for Riscv {
    fn check_func(&self, func: &Func) -> bool {
        for bb in func.iter_bbs() {
            if !BBChecker::check_bb(self, bb) {
                return false;
            }
        }
        true
    }
}

impl BBChecker for Riscv {
    fn check_bb(&self, bb: &Block) -> bool {
        for inst in bb.insts() {
            if !InstChecker::check_inst(self, inst) {
                return false;
            }
        }
        true
    }
}
impl InstChecker for Riscv {
    fn check_inst(&self, inst: &Inst) -> bool {
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
            Inst::Sltu(sltu) => self.check_sltu(sltu),
            Inst::Sgtu(sgut) => self.check_sgtu(sgut),
            Inst::UDiv(udiv) => self.check_udiv(udiv),
            Inst::Feqs(feqs) => self.check_feqs(feqs),
            Inst::Fles(fles) => self.check_fles(fles),
            Inst::Flts(flts) => self.check_flts(flts),
            Inst::Lui(lui) => self.check_lui(lui),
        }
    }
}

impl Riscv {
    fn check_lui(&self, lui: &LuiInst) -> bool {
        matches!(lui.dst(), Operand::Reg(_)) && matches!(lui.src(), Operand::Imm(_))
    }

    fn check_feqs(&self, feqs: &FeqsInst) -> bool {
        matches!(feqs.dst(), Operand::Reg(_))
            && matches!(feqs.lhs(), Operand::Reg(_))
            && matches!(feqs.rhs(), Operand::Reg(_))
    }

    fn check_fles(&self, fles: &FlesInst) -> bool {
        matches!(fles.dst(), Operand::Reg(_))
            && matches!(fles.lhs(), Operand::Reg(_))
            && matches!(fles.rhs(), Operand::Reg(_))
    }

    fn check_flts(&self, flts: &FltsInst) -> bool {
        matches!(flts.dst(), Operand::Reg(_))
            && matches!(flts.lhs(), Operand::Reg(_))
            && matches!(flts.rhs(), Operand::Reg(_))
    }

    fn check_udiv(&self, udiv: &UdivInst) -> bool {
        matches!(udiv.dst(), Operand::Reg(_))
            && matches!(udiv.lhs(), Operand::Reg(_))
            && matches!(udiv.rhs(), Operand::Reg(_))
    }

    fn check_add(&self, add: &AddInst) -> bool {
        matches!(add.dst(), Operand::Reg(_))
            && matches!(add.lhs(), Operand::Reg(_))
            && (matches!(add.rhs(), Operand::Reg(_))
                || Self::check_imm_op_in_i_type_inst(add.rhs()))
    }

    fn check_sltu(&self, sltu: &SltuInst) -> bool {
        matches!(sltu.dst(), Operand::Reg(_))
            && matches!(sltu.lhs(), Operand::Reg(_))
            && matches!(sltu.rhs(), Operand::Reg(_))
    }
    fn check_sgtu(&self, sgtu: &SgtuInst) -> bool {
        matches!(sgtu.dst(), Operand::Reg(_))
            && matches!(sgtu.lhs(), Operand::Reg(_))
            && matches!(sgtu.rhs(), Operand::Reg(_))
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
