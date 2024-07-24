#[macro_export]
macro_rules! ssa2tac_three_usual {
    ($tac_inst_ty:ident /* AddInst */,  $ssa_inst_type:ident /* Add */, $inst:ident, $regs:ident, $reg_gener:ident ) => {{
        let add = downcast_ref::<middle::ir::instruction::binary_inst::$ssa_inst_type>(
            $inst.as_ref().as_ref(),
        );
        let op0 = Self::value_from(add.get_lhs(), $regs)?;
        let op1 = Self::value_from(add.get_rhs(), $regs)?;
        if let (Operand::Reg(op0), Operand::Reg(op1)) = (&op0, &op1) {
            let dst = $reg_gener.gen_virtual_usual_reg();
            $regs.insert(add as *const _ as Address, dst);
            let add_inst = $tac_inst_ty::new(dst.into(), op0.into(), op1.into());
            Ok(vec![add_inst.into()])
        } else if let (Operand::Reg(op0), Operand::Imm(op1)) = (&op0, &op1) {
            let dst = $reg_gener.gen_virtual_usual_reg();
            $regs.insert(add as *const _ as Address, dst);
            let add_inst = $tac_inst_ty::new(dst.into(), op0.into(), op1.into());
            Ok(vec![add_inst.into()])
        } else if let (Operand::Imm(op0), Operand::Reg(op1)) = (&op0, &op1) {
            let dst0 = $reg_gener.gen_virtual_usual_reg();
            let li = LiInst::new(dst0.into(), op0.into());
            let dst1 = $reg_gener.gen_virtual_usual_reg();
            $regs.insert(add as *const _ as Address, dst1);
            let add_inst = $tac_inst_ty::new(dst1.into(), dst0.into(), op1.into());
            Ok(vec![li.into(), add_inst.into()])
        } else {
            // 不太可能两个都是 Imm
            Err(anyhow!("operand type not supported")).with_context(|| context!())
        }
    }};
}

#[macro_export]
macro_rules! ssa2tac_three_float {
    ($inst:ident, $regs:ident, $reg_gener:ident, $ssa_ty:ident, $tac_enum:ident, $tac_inst:ident) => {{
        let dinst =
            downcast_ref::<middle::ir::instruction::binary_inst::$ssa_ty>($inst.as_ref().as_ref());
        let lhs = Self::local_operand_from(dinst.get_lhs(), $regs).with_context(|| context!())?;
        let rhs = Self::local_operand_from(dinst.get_rhs(), $regs).with_context(|| context!())?;
        let dst = $reg_gener.gen_virtual_float_reg();
        $regs.insert(dinst as *const _ as Address, dst);
        let inst = $tac_inst::new(dst.into(), lhs, rhs);
        Ok(vec![Inst::$tac_enum(inst)])
    }};
}
