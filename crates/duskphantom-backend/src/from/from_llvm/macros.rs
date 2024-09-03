// Copyright 2024 Duskphantom Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

#[macro_export]
/// in this macro ,if rhs is constant and out of bound,then it will move it to a reg,then keep on
/// if with AllowSwap attribute ,it will try swap lhs and rhs to make new rhs a constant
macro_rules! llvm2tac_three_usual {
    (AllowSwap; $tac_inst_ty:ident,$inst:ident, $reg_gener:ident,$regs:ident) => {{
        use $crate::irs::*;
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
            return Err(anyhow!("operand type not supported"))
                .with_context(|| duskphantom_utils::context!());
        }
        Ok(ret)
    }};
    (DenySwap; $tac_inst_ty:ident,$inst:ident, $reg_gener:ident,$regs:ident) => {{
        let mut ret: Vec<Inst> = vec![];
        let (lhs, pre_insert) = Self::prepare_usual_lhs(&$inst.operand0, $reg_gener, $regs)?;
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
/// this macro could be use to gen construction of three op inst
/// allowing rhs be constant even out of bound,even unlegal
macro_rules! llvm2tac_r2_c {
    ($tac_inst_ty:ident,$inst:ident, $reg_gener:ident,$regs:ident) => {{
        use $crate::irs::*;
        let mut ret: Vec<Inst> = Vec::new();
        let (lhs, pre_insert) = Self::prepare_usual_lhs(&$inst.operand0, $reg_gener, $regs)?;
        ret.extend(pre_insert);
        let rhs = Self::value_from(&$inst.operand1, $regs)?;
        let dst = $reg_gener.gen_virtual_usual_reg();
        $regs.insert($inst.dest.clone(), dst);
        let mul = $tac_inst_ty::new(dst.into(), lhs, rhs);
        ret.push(mul.into());
        Ok(ret)
    }};
}

#[macro_export]
macro_rules! llvm2tac_three_op_float {
    ($tac_inst_ty:ident,$inst:ident,$reg_gener:ident,$regs:ident,$fmms:ident) => {{
        let mut ret: Vec<Inst> = vec![];
        let (lhs, pre_insert) = Self::prepare_float_lhs(&$inst.operand0, $reg_gener, $regs, $fmms)?;
        ret.extend(pre_insert);
        let (rhs, pre_insert) = Self::prepare_float_rhs(&$inst.operand1, $reg_gener, $regs, $fmms)?;
        ret.extend(pre_insert);
        let dest = $inst.dest.clone();
        let dst = $reg_gener.gen_virtual_reg(false);
        $regs.insert(dest, dst);
        let tac_inst = $tac_inst_ty::new(dst.into(), lhs, rhs);
        ret.push(tac_inst.into());
        Ok(ret)
    }};
}

#[macro_export]
macro_rules! llvm2tac_binary_float {
    ($tac_inst_ty:ident,$inst:ident,$reg_gener:ident,$regs:ident,$fmms:ident) => {{
        let mut ret: Vec<Inst> = vec![];
        let (rhs, pre_insert) = Self::prepare_float_rhs(&$inst.operand, $reg_gener, $regs, $fmms)?;
        ret.extend(pre_insert);
        let dest = $inst.dest.clone();
        let dst = $reg_gener.gen_virtual_reg(false);
        $regs.insert(dest, dst);
        let tac_inst = $tac_inst_ty::new(dst.into(), rhs);
        ret.push(tac_inst.into());
        Ok(ret)
    }};
}
