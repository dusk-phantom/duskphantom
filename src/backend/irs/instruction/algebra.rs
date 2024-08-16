use super::*;

impl_three_op_inst_with_dstmem!(AddInst, "add");
impl_three_op_inst_with_dstmem!(SubInst, "sub");
impl_three_op_inst_with_dstmem!(MulInst, "mul");
impl_three_op_inst!(RemInst, "rem");
impl_three_op_inst!(DivInst, "div");
impl_three_op_inst_with_dstmem!(SllInst, "sll");
impl_three_op_inst!(SrlInst, "srl");
impl_three_op_inst!(SraInst, "sra");
impl_three_op_inst!(AndInst, "and");
impl_two_op_inst!(NotInst, "not");
impl_three_op_inst!(OrInst, "or");
impl_three_op_inst!(XorInst, "xor");
impl_three_op_inst!(UdivInst, "divuw");

// 实现比较指令
impl_three_op_inst!(SltInst, "slt");
impl_three_op_inst!(SltuInst, "sltu");
impl_three_op_inst!(SgtuInst, "sgtu");
impl_two_op_inst!(SnezInst, "snez");
impl_two_op_inst!(SeqzInst, "seqz");

impl_three_op_inst!(FeqsInst, "feq.s");
impl_three_op_inst!(FlesInst, "fle.s");
impl_three_op_inst!(FltsInst, "flt.s");

impl_two_op_inst!(NegInst, "neg");
impl_two_op_inst!(MvInst, "mv");

////////////////////////////////////////////////////////////////////////
/// 以下是具体指令类型 与 Inst 的转换
////////////////////////////////////////////////////////////////////////
mod c {
    use super::*;
    // for algebraic operation
    impl_inst_convert!(AddInst, Add);
    impl_inst_convert!(SubInst, Sub);
    impl_inst_convert!(MulInst, Mul);
    impl_inst_convert!(RemInst, Rem);
    impl_inst_convert!(DivInst, Div);
    impl_inst_convert!(NegInst, Neg);

    // for bit count operation
    impl_inst_convert!(AndInst, And);
    impl_inst_convert!(OrInst, Or);
    impl_inst_convert!(XorInst, Xor);
    impl_inst_convert!(SllInst, Sll);
    impl_inst_convert!(SrlInst, Srl);
    impl_inst_convert!(SraInst, SRA);
    impl_inst_convert!(NotInst, Not);

    // for comparison
    impl_inst_convert!(SltInst, Slt);
    impl_inst_convert!(SltuInst, Sltu);
    impl_inst_convert!(SgtuInst, Sgtu);
    impl_inst_convert!(SeqzInst, Seqz);
    impl_inst_convert!(SnezInst, Snez);
    impl_inst_convert!(FeqsInst, Feqs);
    impl_inst_convert!(FlesInst, Fles);
    impl_inst_convert!(FltsInst, Flts);
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_gem_asm_add() {
        let add = AddInst::new(REG_A0.into(), REG_A1.into(), REG_A2.into());
        assert_eq!(add.gen_asm(), "addw a0,a1,a2");
        let addiw = AddInst::new(REG_A0.into(), REG_A1.into(), 1.into());
        assert_eq!(addiw.gen_asm(), "addiw a0,a1,1");
        let addi = AddInst::new(REG_A0.into(), REG_A1.into(), 1.into()).with_8byte();
        assert_eq!(addi.gen_asm(), "addi a0,a1,1");
        let fadd = AddInst::new(REG_FA0.into(), REG_FA1.into(), REG_FA2.into());
        assert_eq!(fadd.gen_asm(), "fadd.s fa0,fa1,fa2");
    }
    #[test]
    fn test_gem_asm_sub() {
        let sub = SubInst::new(REG_A0.into(), REG_A1.into(), REG_A2.into());
        assert_eq!(sub.gen_asm(), "subw a0,a1,a2");
        let subi = SubInst::new(REG_A0.into(), REG_A1.into(), 1.into());
        assert!(!checker::InstChecker::check_inst(
            &checker::Riscv,
            &subi.into()
        ),);
        let fsub = SubInst::new(REG_FA0.into(), REG_FA1.into(), REG_FA2.into());
        assert_eq!(fsub.gen_asm(), "fsub.s fa0,fa1,fa2");
    }
}
