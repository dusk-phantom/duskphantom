#[macro_export]
macro_rules! llvm2tac_three_op_usual {
    (AllowSwap; $tac_inst_ty:ident,$inst:ident, $reg_gener:ident,$regs:ident) => {{
        use $crate::backend::irs::*;
        let mut ret: Vec<Inst> = vec![];
        let op0 = &Self::value_from(&$inst.operand0, $regs)?;
        let op1 = &Self::value_from(&$inst.operand1, $regs)?;
        let dest = $inst.dest.clone();
        let dst = $reg_gener.gen_virtual_reg(true);
        $regs.insert(dest, dst);
        if let (Operand::Reg(op0), Operand::Reg(op1)) = (op0, op1) {
            let tac_inst = $tac_inst_ty::new(dst.into(), op0.into(), op1.into());
            ret.push(tac_inst.into());
        } else if let (Operand::Reg(op0), Operand::Imm(_)) = (op0, op1) {
            let (rhs, pre_insert) = Self::prepare_usual_rhs(&$inst.operand1, $reg_gener, $regs)?;
            ret.extend(pre_insert);
            let tac_inst = $tac_inst_ty::new(dst.into(), op0.into(), rhs);
            ret.push(tac_inst.into());
        } else if let (Operand::Imm(_), Operand::Reg(op1)) = (op0, op1) {
            let (rhs, pre_insert) = Self::prepare_usual_rhs(&$inst.operand0, $reg_gener, $regs)?;
            ret.extend(pre_insert);
            let tac_inst = $tac_inst_ty::new(dst.into(), op1.into(), rhs);
            ret.extend([tac_inst.into()]);
        } else {
            return Err(anyhow!("operand type not supported")).with_context(|| context!());
        }
        Ok(ret)
    }};
    (DenySwap; $tac_inst_ty:ident,$inst:ident, $reg_gener:ident,$regs:ident) => {{
        let mut ret: Vec<Inst> = vec![];
        let (lhs, pre_insert) = Self::prepare_lhs(&$inst.operand0, $reg_gener, $regs)?;
        ret.extend(pre_insert);
        let (rhs, pre_insert) = Self::prepare_usual_rhs(&$inst.operand1, $reg_gener, $regs)?;
        ret.extend(pre_insert);
        let dest = $inst.dest.clone();
        let dst = $reg_gener.gen_virtual_reg(true);
        $regs.insert(dest, dst);
        let tac_inst = $tac_inst_ty::new(dst.into(), lhs, rhs);
        ret.push(tac_inst.into());
        Ok(ret)
    }};
}

#[macro_export]
macro_rules! llvm2tac_three_op_float {
    () => {
        unimplemented!()
    };
}
