use super::*;

impl IRBuilder {
    pub fn build_gep_inst(
        gep: &middle::ir::instruction::memory_op_inst::GetElementPtr,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
        stack_slots: &HashMap<Address, StackSlot>,
    ) -> Result<Vec<Inst>> {
        let mut ret = Vec::new();

        let idxes = gep.get_index();
        let ty = gep.get_ptr().get_type();
        // println!("{}", gep);
        // dbg!(&ty);
        let (_, ofst, prepare) =
            Self::__cal_offset(&ty, idxes, reg_gener, regs).with_context(|| context!())?;
        ret.extend(prepare);
        // regs.insert(gep as *const _ as usize, ofst);

        // // println!("gep: {}", gep);

        // /* ---------- 计算 offset ---------- */
        // let idxes = gep.get_index();
        // let capas = {
        //     let mut v = Self::_cal_capas_rev(&gep.element_type);
        //     v.reverse();
        //     v
        // };
        // let (ofst, prepare) =
        //     Self::_cal_offset(&capas, idxes, reg_gener, regs).with_context(|| context!())?;
        // ret.extend(prepare);
        let _mid = reg_gener.gen_virtual_usual_reg();
        let slli = SllInst::new(_mid.into(), ofst.into(), (2).into()); // FIXME sysy 的数据都是 4Byte, 但是我感觉我这里不严谨
        ret.push(slli.into());

        // /* ---------- base ---------- */
        let ptr = gep.get_ptr();
        let base: Reg =
            match Self::address_from(ptr, regs, stack_slots).with_context(|| context!())? {
                Operand::Reg(reg) => reg,
                Operand::StackSlot(slot) => {
                    let addr = reg_gener.gen_virtual_usual_reg();
                    let laddr = LocalAddr::new(addr, slot);
                    ret.push(laddr.into());
                    addr
                }
                Operand::Label(label) => {
                    let dst = reg_gener.gen_virtual_usual_reg();
                    let lla = LlaInst::new(dst, label);
                    ret.push(lla.into());
                    dst
                }
                _ => unimplemented!(), // Fmm(_) Imm(_)
            };
        let dst = reg_gener.gen_virtual_usual_reg();
        let add = AddInst::new(dst.into(), base.into(), _mid.into());
        ret.push(add.into());
        regs.insert(gep as *const _ as usize, dst);
        Ok(ret)
    }

    fn __cal_offset(
        ty: &middle::ir::ValueType,
        idxes: &[middle::ir::Operand],
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Address, Reg>,
    ) -> Result<(
        usize, /* 部分阶乘 */
        Reg,   /* 部分结果 */
        Vec<Inst>,
    )> {
        let mut ret = Vec::new();
        match ty {
            middle::ir::ValueType::Void => {
                Err(anyhow!("gep can't be void: {}", ty)).with_context(|| context!())
            }
            middle::ir::ValueType::SignedChar
            | middle::ir::ValueType::Int
            | middle::ir::ValueType::Bool => todo!(),
            middle::ir::ValueType::Float => todo!(),
            middle::ir::ValueType::Array(ty, sz) => {
                let (idx, prepare) =
                    Self::prepare_rs1_i(&idxes[0], reg_gener, regs).with_context(|| context!())?;
                ret.extend(prepare);
                if idxes.len() > 1 {
                    let (factor, acc, prepare) =
                        Self::__cal_offset(ty, &idxes[1..], reg_gener, regs)
                            .with_context(|| context!())?;
                    ret.extend(prepare);
                    let factor = sz * factor;
                    let dst0 = reg_gener.gen_virtual_usual_reg();
                    let li = LiInst::new(dst0.into(), (factor as i64).into());
                    ret.push(li.into());
                    let dst1 = reg_gener.gen_virtual_usual_reg();
                    let mul = MulInst::new(dst1.into(), idx.clone(), dst0.into());
                    ret.push(mul.into());
                    let _acc = reg_gener.gen_virtual_usual_reg();
                    let add = AddInst::new(_acc.into(), acc.into(), dst1.into());
                    ret.push(add.into());
                    Ok((factor, _acc, ret))
                } else {
                    let factor = Self::_cal_capas_factor(ty).with_context(|| context!())?;
                    let dst0 = reg_gener.gen_virtual_usual_reg();
                    let li = LiInst::new(dst0.into(), (factor as i64).into());
                    ret.push(li.into());
                    let dst1 = reg_gener.gen_virtual_usual_reg();
                    let mul = MulInst::new(dst1.into(), idx.clone(), dst0.into());
                    ret.push(mul.into());
                    Ok((factor, dst1, ret))
                }
            }
            middle::ir::ValueType::Pointer(poi) => {
                let (idx, prepare) =
                    Self::prepare_rs1_i(&idxes[0], reg_gener, regs).with_context(|| context!())?;
                ret.extend(prepare);
                // dbg!(poi);
                if idxes.len() > 1 {
                    let (factor /* 指向的数组的大小 */, acc, prepare) =
                        Self::__cal_offset(poi, &idxes[1..], reg_gener, regs)
                            .with_context(|| context!())?;
                    ret.extend(prepare);
                    let dst0 = reg_gener.gen_virtual_usual_reg();
                    let li = LiInst::new(dst0.into(), (factor as i64).into());
                    ret.push(li.into());
                    let dst1 = reg_gener.gen_virtual_usual_reg();
                    let mul = MulInst::new(dst1.into(), idx.clone(), dst0.into());
                    ret.push(mul.into());
                    let _acc = reg_gener.gen_virtual_usual_reg();
                    let add = AddInst::new(_acc.into(), acc.into(), dst1.into());
                    ret.push(add.into());
                    Ok((factor, _acc, ret))
                } else {
                    let factor = Self::_cal_capas_factor(poi).with_context(|| context!())?;
                    let dst0 = reg_gener.gen_virtual_usual_reg();
                    let li = LiInst::new(dst0.into(), (factor as i64).into());
                    ret.push(li.into());
                    let dst1 = reg_gener.gen_virtual_usual_reg();
                    let mul = MulInst::new(dst1.into(), idx.clone(), dst0.into());
                    ret.push(mul.into());
                    Ok((factor, dst1, ret))
                }
                // 判断下面有没有了,
            }
        }
    }

    pub fn _cal_capas_factor(ty: &middle::ir::ValueType) -> Result<usize> {
        match ty {
            middle::ir::ValueType::Void => {
                Err(anyhow!("gep can't be void: {}", ty)).with_context(|| context!())
            }
            middle::ir::ValueType::Pointer(_) => todo!(),
            middle::ir::ValueType::SignedChar
            | middle::ir::ValueType::Int
            | middle::ir::ValueType::Float
            | middle::ir::ValueType::Bool => Ok(1),
            middle::ir::ValueType::Array(ty, sz) => {
                let ty = ty.as_ref();
                Ok(sz * Self::_cal_capas_factor(ty).with_context(|| context!())?)
            }
        }
    }
}
