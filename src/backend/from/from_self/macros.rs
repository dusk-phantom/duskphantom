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
        let addi_inst = $tac_inst_ty::new(dst.into(), op0.into(), op1);
        insts.push(addi_inst.into());
        Ok(insts)
    }};
}

#[macro_export]
macro_rules! ssa2tac_three_float {
    ($tac_inst_ty:ident /* AddInst */,  $ssa_inst_type:ident /* FAdd */, $inst:ident, $regs:ident, $reg_gener:ident, $fmms:ident) => {{
        let mut insts = Vec::new();
        let fadd = downcast_ref::<middle::ir::instruction::binary_inst::$ssa_inst_type>(
            $inst.as_ref().as_ref(),
        );
        let (op0, prepare) = Self::prepare_f(fadd.get_lhs(), $reg_gener, $regs, $fmms)?;
        insts.extend(prepare);
        let (op1, prepare) = Self::prepare_f(fadd.get_rhs(), $reg_gener, $regs, $fmms)?;
        insts.extend(prepare);
        let dst0 = $reg_gener.gen_virtual_float_reg();
        let fadd_inst = $tac_inst_ty::new(dst0.into(), op0, op1);
        $regs.insert(fadd as *const _ as Address, dst0);
        insts.push(fadd_inst.into());
        Ok(insts)
    }};
}
