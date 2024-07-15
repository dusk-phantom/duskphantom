#[macro_export]
macro_rules! llvm2tac_binary_usual {
    ($tac_inst_ty:ident,$inst:ident, $reg_gener:ident,$regs:ident) => {{
        let op0 = &Self::value_from(&$inst.operand0, $regs)?;
        let op1 = &Self::value_from(&$inst.operand1, $regs)?;
        let dest = $inst.dest.clone();
        if let (Operand::Reg(op0), Operand::Reg(op1)) = (op0, op1) {
            let dst = $reg_gener.gen_virtual_reg(op0.is_usual());
            $regs.insert(dest, dst);
            let add_inst = $tac_inst_ty::new(dst.into(), op0.into(), op1.into());
            Ok(vec![add_inst.into()])
        } else if let (Operand::Reg(op0), Operand::Imm(op1)) = (op0, op1) {
            let dst = $reg_gener.gen_virtual_reg(op0.is_usual());
            $regs.insert(dest, dst);
            let add_inst = $tac_inst_ty::new(dst.into(), op0.into(), op1.into());
            Ok(vec![add_inst.into()])
        } else if let (Operand::Imm(op0), Operand::Reg(op1)) = (op0, op1) {
            let dst = $reg_gener.gen_virtual_reg(op1.is_usual());
            $regs.insert(dest, dst);
            let add_inst = $tac_inst_ty::new(dst.into(), op0.into(), op1.into());
            Ok(vec![add_inst.into()])
        } else {
            Err(anyhow!("operand type not supported")).with_context(|| context!())
        }
    }};
}
