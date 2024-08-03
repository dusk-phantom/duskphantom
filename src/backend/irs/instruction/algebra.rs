use super::*;

impl_three_op_inst!(AddInst, "add");
impl_three_op_inst!(SubInst, "sub");
impl_three_op_inst!(MulInst, "mul");
impl_three_op_inst!(RemInst, "rem");
impl_three_op_inst!(DivInst, "div");
impl_three_op_inst!(SllInst, "sll");
impl_three_op_inst!(SrlInst, "srl");
impl_three_op_inst!(SraInst, "sra");
impl_three_op_inst!(AndInst, "and");
impl_two_op_inst!(NotInst, "not");
impl_three_op_inst!(OrInst, "or");
impl_three_op_inst!(XorInst, "xor");
impl_three_op_inst!(UdivInst, "divu");

// 实现比较指令
impl_three_op_inst!(SltInst, "slt");
impl_three_op_inst!(SltuInst, "sltu");
impl_three_op_inst!(SgtuInst, "sgtu");
impl_two_op_inst!(SnezInst, "snez");
impl_two_op_inst!(SeqzInst, "seqz");

impl_two_op_inst!(NegInst, "neg");
impl_two_op_inst!(MvInst, "mv");

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_gem_asm_add() {
        let add = AddInst::new(REG_A0.into(), REG_A1.into(), REG_A2.into());
        assert_eq!(add.gen_asm(), "add a0,a1,a2");
        let addi = AddInst::new(REG_A0.into(), REG_A1.into(), 1.into());
        assert_eq!(addi.gen_asm(), "addi a0,a1,1");
        let fadd = AddInst::new(REG_FA0.into(), REG_FA1.into(), REG_FA2.into());
        assert_eq!(fadd.gen_asm(), "fadd.s fa0,fa1,fa2");
    }
    #[test]
    fn test_gem_asm_sub() {
        let sub = SubInst::new(REG_A0.into(), REG_A1.into(), REG_A2.into());
        assert_eq!(sub.gen_asm(), "sub a0,a1,a2");
        let subi = SubInst::new(REG_A0.into(), REG_A1.into(), 1.into());
        assert!(!checker::InstChecker::check_valid(
            &checker::Riscv,
            &subi.into()
        ),);
        let fsub = SubInst::new(REG_FA0.into(), REG_FA1.into(), REG_FA2.into());
        assert_eq!(fsub.gen_asm(), "fsub.s fa0,fa1,fa2");
    }
}
