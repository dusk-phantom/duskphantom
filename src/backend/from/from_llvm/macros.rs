#[macro_export]
macro_rules! llvm2tac_three_op_usual {
    ($tac_inst_ty:ident,$inst:ident, $reg_gener:ident,$regs:ident) => {{
        llvm2tac_three_op_usual!(DenySwap; $tac_inst_ty, $inst, $reg_gener,$regs)
    }};
    (AllowSwap; $tac_inst_ty:ident,$inst:ident, $reg_gener:ident,$regs:ident) => {{
        let mut f=||->Result<Vec<Inst>>{
            let mut ret:Vec<Inst>=vec![];
            let op0 = &Self::value_from(&$inst.operand0, $regs)?;
            let op1 = &Self::value_from(&$inst.operand1, $regs)?;
            let dest = $inst.dest.clone();
            let dst = $reg_gener.gen_virtual_reg(true);
            $regs.insert(dest, dst);
            if let (Operand::Reg(op0), Operand::Reg(op1)) = (op0, op1) {
                let tac_inst = $tac_inst_ty::new(dst.into(), op0.into(), op1.into());
                ret.push(tac_inst.into());
            } else if let (Operand::Reg(op0), Operand::Imm(op1)) = (op0, op1) {
                let tac_inst = $tac_inst_ty::new(dst.into(), op0.into(), op1.into());
                ret.push(tac_inst.into());
            } else if let (Operand::Imm(op0), Operand::Reg(op1)) = (op0, op1) {
                let tac_inst = $tac_inst_ty::new(dst.into(), op1.into(), op0.into());
                ret.extend([tac_inst.into()]);
            } else {
                return Err(anyhow!("operand type not supported")).with_context(|| context!());
            }
            Ok(ret)
        };
        f()
    }};
    (DenySwap; $tac_inst_ty:ident,$inst:ident, $reg_gener:ident,$regs:ident) => {{
        let mut ret:Vec<Inst>=vec![];
        let (lhs,pre_insert)=Self::prepare_lhs(&$inst.operand0,$reg_gener,$regs)?;
        ret.extend(pre_insert);
        let (rhs,pre_insert)=Self::prepare_rhs(&$inst.operand1,$reg_gener,$regs)?;
        ret.extend(pre_insert);
        let dest = $inst.dest.clone();
        let dst = $reg_gener.gen_virtual_reg(true);
        $regs.insert(dest, dst);
        let tac_inst=$tac_inst_ty::new(dst.into(),lhs,rhs);
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
