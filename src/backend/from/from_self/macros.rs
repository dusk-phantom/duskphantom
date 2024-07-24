#[macro_export]
macro_rules! ssa2tac_three_usual_Rtype {
    ($tac_inst_ty:ident /* AddInst */,  $ssa_inst_type:ident /* Add */, $inst:ident, $regs:ident, $reg_gener:ident) => {{
        let mut insts = Vec::new();
        let sub = downcast_ref::<middle::ir::instruction::binary_inst::$ssa_inst_type>(
            $inst.as_ref().as_ref(),
        );
        let (op0, prepare) = Self::prepare_rs1_i(sub.get_lhs(), $reg_gener, $regs)?;
        insts.extend(prepare);
        // 这里使用的是 prepare_lhs, mul rs1/rs2 都只能是 Reg, 这个很重要
        let (op1, prepare) = Self::prepare_rs1_i(sub.get_rhs(), $reg_gener, $regs)?;
        insts.extend(prepare);
        let dst = $reg_gener.gen_virtual_usual_reg();
        $regs.insert(sub as *const _ as Address, dst);
        let sub_inst = $tac_inst_ty::new(dst.into(), op0, op1);
        insts.push(sub_inst.into());
        Ok(insts)
    }};
}

#[macro_export]
macro_rules! ssa2tac_three_usual_Itype {
    ($tac_inst_ty:ident /* AddInst */,  $ssa_inst_type:ident /* Add */, $inst:ident, $regs:ident, $reg_gener:ident) => {{
        let mut insts = Vec::new();
        let addi = downcast_ref::<middle::ir::instruction::binary_inst::$ssa_inst_type>(
            $inst.as_ref().as_ref(),
        );
        let (op0, prepare) = Self::prepare_rs1_i(addi.get_lhs(), $reg_gener, $regs)?;
        insts.extend(prepare);
        let (op1, prepare) = Self::prepare_rs2_i(addi.get_rhs(), $reg_gener, $regs)?;
        insts.extend(prepare);
        let dst = $reg_gener.gen_virtual_usual_reg();
        $regs.insert(addi as *const _ as Address, dst);
        let addi_inst = $tac_inst_ty::new(dst.into(), op0, op1);
        insts.push(addi_inst.into());
        Ok(insts)
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
