use crate::fprintln;

use super::*;

/** @brief GetElementPtrInst
 */
impl IRBuilder {
    pub fn build_gep_inst(
        gep: &middle::ir::instruction::memory_op_inst::GetElementPtr,
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
        stack_slots: &HashMap<Address, StackSlot>,
    ) -> Result<Vec<Inst>> {
        let mut ret = Vec::new();
        fprintln!("log/build_gep_inst.log";'a';"gep:{}",gep);

        let idxes = gep.get_index();
        let ty = gep.get_ptr().get_type();
        let (_, ofst, prepare) =
            Self::_cal_offset(&ty, idxes, reg_gener, regs).with_context(|| context!())?;
        ret.extend(prepare);
        let _mid = reg_gener.gen_virtual_usual_reg();

        fprintln!("log/build_gep_inst.log";'a';"ofst:{:?}",ofst);

        let slli = SllInst::new(_mid.into(), ofst.into(), (2).into()).with_8byte(); // sysy 的数据都是 4Byte
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

        fprintln!("log/build_gep_inst.log";'a';"base:{:?}", base);

        let dst = reg_gener.gen_virtual_usual_reg();
        let add = AddInst::new(dst.into(), base.into(), _mid.into()).with_8byte();

        fprintln!("log/build_gep_inst.log";'a';"final:{:?}", dst);

        ret.push(add.into());
        regs.insert(gep as *const _ as usize, dst);
        Ok(ret)
    }

    fn _cal_offset(
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
        if idxes.is_empty() {
            let factor = Self::_cal_capas_factor(ty).with_context(|| context!())?;
            // println!("ty={}, factor={}", ty, factor);
            return Ok((factor, REG_ZERO, ret));
        }
        match ty {
            middle::ir::ValueType::Void => {
                Err(anyhow!("gep can't be void: {}", ty)).with_context(|| context!())
            }
            middle::ir::ValueType::SignedChar
            | middle::ir::ValueType::Int
            | middle::ir::ValueType::Bool
            | middle::ir::ValueType::Float => {
                assert!(idxes.len() == 1); // 这种情况是 idxes 和 types 同时耗尽
                let (idx, prepare) =
                    Self::prepare_rs1_i(&idxes[0], reg_gener, regs).with_context(|| context!())?;
                ret.extend(prepare);
                // println!("{}", ty);
                Ok((1, idx, ret))
            }
            middle::ir::ValueType::Array(ty, sz) => {
                // 这里 ty 是拿到当前数组的类型
                let (idx, prepare) =
                    Self::prepare_rs1_i(&idxes[0], reg_gener, regs).with_context(|| context!())?;
                ret.extend(prepare);
                let (_factor, _acc, prepare) = Self::_cal_offset(ty, &idxes[1..], reg_gener, regs)
                    .with_context(|| context!())?;
                ret.extend(prepare);
                let factor = sz * _factor; // 当前类型的 sizeof
                let fac = reg_gener.gen_virtual_usual_reg();
                let li = LiInst::new(fac.into(), (factor as i64).into()); // 部分阶乘
                ret.push(li.into());
                let part = reg_gener.gen_virtual_usual_reg(); // 部分积
                let mul = MulInst::new(part.into(), idx.into(), fac.into()).with_8byte();
                ret.push(mul.into());
                let acc = reg_gener.gen_virtual_usual_reg(); // 部分结果
                let add = AddInst::new(acc.into(), _acc.into(), part.into()).with_8byte();
                ret.push(add.into());
                Ok((factor, acc, ret))
            }
            // %getelementptr_85 = getelementptr i32, ptr %getelementptr_84, i32 0
            // %getelementptr_57 = getelementptr [2 x i32], ptr %getelementptr_38, i32 3
            // %getelementptr_58 = getelementptr [2 x i32], ptr %getelementptr_57, i32 0, i32 0
            // 应该这么说, gep 的第一层永远是 ptr
            middle::ir::ValueType::Pointer(poi) => Self::_cal_offset(poi, idxes, reg_gener, regs),
        }
    }
}

/** @brief 一些针对数组的 辅助函数
 */
impl IRBuilder {
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
