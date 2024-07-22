use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};

use crate::backend::{Operand, Reg, StackSlot};

use crate::context;

use crate::middle;
use crate::middle::ir::Instruction;
use crate::utils::mem::ObjPtr;

use super::*;
use builder::IRBuilder;

impl IRBuilder {
    pub fn is_ty_int(ty: &middle::ir::ValueType) -> bool {
        matches!(ty, middle::ir::ValueType::Int)
    }
    pub fn is_ty_float(ty: &middle::ir::ValueType) -> bool {
        matches!(ty, middle::ir::ValueType::Float)
    }

    pub fn stack_slot_from(
        operand: &middle::ir::Operand,
        stack_slots: &HashMap<Address, StackSlot>,
    ) -> Result<Operand> {
        Ok(match operand {
            middle::ir::Operand::Instruction(instr) => stack_slots
                .get(&(instr.as_ref().as_ref() as *const dyn Instruction as *const () as Address))
                .ok_or(anyhow!(
                    "stack slot not found {}",
                    (instr.as_ref().as_ref() as *const dyn Instruction as *const () as Address)
                ))
                .with_context(|| context!())?
                .into(), // 这个 into 将 stackslot -> operand
            _ => {
                /* Constant, Global */
                /* Parameter 只有 void, int, float 三种类型 */
                return Err(anyhow!("operand is not local var:{}", operand))
                    .with_context(|| context!());
            }
        })
    }

    /// 找到指令的 output 对应的 reg, 查表 !
    /// 在这里好像看不出来是 int 还是 float
    pub fn local_var_from(
        instr: &ObjPtr<Box<dyn Instruction>>,
        regs: &HashMap<Address, Reg>,
    ) -> Result<Operand> {
        let addr = instr.as_ref().as_ref() as *const dyn Instruction as *const () as Address;
        let reg = regs
            .get(&addr)
            .ok_or(anyhow!(
                "local var not found {}",
                instr.as_ref().as_ref() as *const dyn Instruction as *const () as Address
            ))
            .with_context(|| context!())?;
        Ok((*reg).into())
    }
    pub fn const_from(con: &middle::ir::Constant) -> Result<Operand> {
        Ok(match con {
            middle::ir::Constant::Int(val) => Operand::Imm((*val as i64).into()),
            middle::ir::Constant::Float(fla) => Operand::Fmm((*fla as f64).into()),
            middle::ir::Constant::Bool(boo) => Operand::Imm((*boo as i64).into()),
            middle::ir::Constant::SignedChar(sig) => Operand::Imm((*sig as i64).into()),
            middle::ir::Constant::Array(_) => {
                return Err(anyhow!("const_from operand cann't not be array:{}", con))
                    .with_context(|| context!())
            }
        })
    }

    /// 因为 build_entry 的时候, 就已经把参数 mv <虚拟寄存器>, <param> 了
    pub fn parameter_from(
        param: &middle::ir::Parameter,
        regs: &HashMap<Address, Reg>,
    ) -> Result<Operand> {
        let addr = param as *const _ as Address;
        let reg = regs
            .get(&addr)
            .ok_or(anyhow!(
                "local var not found {}",
                param as *const _ as Address
            ))
            .with_context(|| context!())?;
        Ok((*reg).into())
    }

    /// 获取 basic block 的 label
    #[inline]
    pub fn label_name_from(bb: &ObjPtr<middle::ir::BasicBlock>) -> Result<String> {
        Ok(format!(".LBB{}", bb.as_ref() as *const _ as Address))
    }

    /// 要不是 instruction 的输出, 要不是 constant 要不是 parameter
    /// 这个只是将 instruction 和 constant 包装成 Operand
    /// 里面不会出现 asm 的输出
    pub fn value_from(
        operand: &middle::ir::Operand,
        regs: &HashMap<Address, Reg>,
    ) -> Result<Operand> {
        match operand {
            middle::ir::Operand::Constant(con) => Self::const_from(con),
            middle::ir::Operand::Parameter(param) => Self::parameter_from(param, regs),
            middle::ir::Operand::Instruction(instr) => Self::local_var_from(instr, regs),
            middle::ir::Operand::Global(glo) => Err(anyhow!(
                "local_operand_from operand cann't not be global:{}",
                glo
            ))
            .with_context(|| context!()),
        }
    }

    pub fn pointer_from(
        operand: &middle::ir::Operand,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Reg> {
        match operand {
            middle::ir::Operand::Parameter(param) => {
                let param = param.as_ref();
                match param.value_type {
                    middle::ir::ValueType::Array(_, _) | middle::ir::ValueType::Pointer(_) => {
                        let addr = param as *const _ as Address;
                        let reg = regs
                            .get(&addr)
                            .ok_or(anyhow!(
                                "local var not found {}",
                                param as *const _ as Address
                            ))
                            .with_context(|| context!())?;
                        Ok(*reg)
                    }
                    _ => Err(anyhow!(
                        "it is impossible to load from a void/bool/int/float paramter: {}",
                        operand
                    ))
                    .with_context(|| context!()),
                }
            }
            middle::ir::Operand::Instruction(_) => {
                unimplemented!() /* FIXME 这应该是一个 UB */
            }
            middle::ir::Operand::Constant(_) => Err(anyhow!(
                "it is impossible to load from a constant: {}",
                operand
            ))
            .with_context(|| context!()),
            middle::ir::Operand::Global(_) => Err(anyhow!(
                "global have been processed in global_from: {}",
                operand
            ))
            .with_context(|| context!()),
        }
    }

    /// 我们的 global/函数名 都是来自于中端的 name 的, 其他的 id 来自于中端的地址
    #[inline]
    pub fn global_from(operand: &middle::ir::Operand) -> Result<String> {
        // TODO
        match operand {
            middle::ir::Operand::Global(glo) => {
                let glo = glo.as_ref();
                let label = glo.name.clone();
                Ok(label)
            }
            _ => Err(anyhow!("not a global var:{}", operand)).with_context(|| context!()),
        }
    }
}
