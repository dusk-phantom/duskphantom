use super::*;

use super::super::prog::Program;
use crate::clang_frontend;
use crate::errors::BackendError;
use llvm_ir::{Constant, Name};
use std::collections::HashMap;

#[cfg(feature = "clang_enabled")]
#[allow(unused)]
pub fn gen_from_clang(program: &clang_frontend::Program) -> Result<Program, BackendError> {
    let mut global_vars = Vec::new();
    let mut funcs = Vec::new();
    let llvm = &program.llvm;
    // dbg!(&llvm.types);
    for global_var in &llvm.global_vars {
        // dbg!(&global_var);
        let name = &global_var.name.to_string()[1..];
        if let Some(init) = &global_var.initializer {
            // dbg!(&init);
            let c = init.as_ref().to_owned();
            match c {
                Constant::Int { bits, value } => {
                    let var = var::Var::Prim(var::PrimVar::IntVar(var::IntVar {
                        name: name.to_string(),
                        init: Some(value as i32),
                        is_const: false,
                    }));
                    global_vars.push(var);
                }
                Constant::Float(f) => match f {
                    llvm_ir::constant::Float::Single(f) => {
                        let var = var::Var::Prim(var::PrimVar::FloatVar(var::FloatVar {
                            name: name.to_string(),
                            init: Some(f),
                            is_const: false,
                        }));
                        global_vars.push(var);
                    }
                    llvm_ir::constant::Float::Double(f) => {
                        unimplemented!("double float");
                        // let var = var::Var::Prim(var::PrimVar::FloatVar(var::FloatVar {
                        //     name: name.to_string(),
                        //     init: Some(f),
                        //     is_const: false,
                        // }));
                        // global_vars.push(var);
                    }
                    _ => {
                        unreachable!();
                    }
                },
                _ => (),
            }
        }
    }
    // dbg!(&global_vars);

    for f in &llvm.functions {
        // dbg!(&f);
        let args: Vec<String> = f.parameters.iter().map(|p| p.name.to_string()).collect();

        let ret_ty = &f.return_type;
        let mut stack_allocator = StackAllocator::new();
        let mut stack_slots: HashMap<Name, StackSlot> = HashMap::new();
        let mut bb = f.basic_blocks.first().expect("func must have entry");
        let entry = build_bb(bb, &mut stack_allocator, &mut stack_slots)?;
        let mut m_f = Func::new(f.name.to_string(), args, entry);
        // dbg!(&ret_ty);
        for bb in &f.basic_blocks[1..] {
            let m_bb = build_bb(bb, &mut stack_allocator, &mut stack_slots)?;
            m_f.push_bb(m_bb);
        }
        // count stack size,
        let stack_size = stack_allocator.allocated();
        // align to 16
        let stack_size = if stack_size % 16 == 0 {
            stack_size
        } else {
            stack_size - stack_size % 16 + 16
        };
        funcs.push(m_f);
    }
    let mdl = module::Module {
        name: "main".to_string(),
        entry: Some("main".to_string()),
        global: global_vars,
        funcs,
    };
    Ok(prog::Program {
        entry: None,
        modules: vec![mdl],
    })
}

#[allow(unused)]
pub fn build_instruction(
    inst: &llvm_ir::Instruction,
    stack_allocator: &mut StackAllocator,
    stack_slots: &mut HashMap<Name, StackSlot>,
) -> Result<Vec<Inst>, BackendError> {
    match inst {
        llvm_ir::Instruction::Add(_) => todo!(),
        llvm_ir::Instruction::Sub(_) => todo!(),
        llvm_ir::Instruction::Mul(_) => todo!(),
        llvm_ir::Instruction::UDiv(_) => todo!(),
        llvm_ir::Instruction::SDiv(_) => todo!(),
        llvm_ir::Instruction::URem(_) => todo!(),
        llvm_ir::Instruction::SRem(_) => todo!(),
        llvm_ir::Instruction::And(_) => todo!(),
        llvm_ir::Instruction::Or(_) => todo!(),
        llvm_ir::Instruction::Xor(_) => todo!(),
        llvm_ir::Instruction::Shl(_) => todo!(),
        llvm_ir::Instruction::LShr(_) => todo!(),
        llvm_ir::Instruction::AShr(_) => todo!(),
        llvm_ir::Instruction::FAdd(_) => todo!(),
        llvm_ir::Instruction::FSub(_) => todo!(),
        llvm_ir::Instruction::FMul(_) => todo!(),
        llvm_ir::Instruction::FDiv(_) => todo!(),
        llvm_ir::Instruction::FRem(_) => todo!(),
        llvm_ir::Instruction::FNeg(_) => todo!(),
        llvm_ir::Instruction::ExtractElement(_) => todo!(),
        llvm_ir::Instruction::InsertElement(_) => todo!(),
        llvm_ir::Instruction::ShuffleVector(_) => todo!(),
        llvm_ir::Instruction::ExtractValue(_) => todo!(),
        llvm_ir::Instruction::InsertValue(_) => todo!(),
        llvm_ir::Instruction::Alloca(alloca) => {
            build_alloca_inst(alloca, stack_allocator, stack_slots)
        }
        llvm_ir::Instruction::Load(_) => todo!(),
        llvm_ir::Instruction::Store(store) => build_store_inst(store, stack_allocator, stack_slots),
        llvm_ir::Instruction::Fence(_) => todo!(),
        llvm_ir::Instruction::CmpXchg(_) => todo!(),
        llvm_ir::Instruction::AtomicRMW(_) => todo!(),
        llvm_ir::Instruction::GetElementPtr(_) => todo!(),
        llvm_ir::Instruction::Trunc(_) => todo!(),
        llvm_ir::Instruction::ZExt(_) => todo!(),
        llvm_ir::Instruction::SExt(_) => todo!(),
        llvm_ir::Instruction::FPTrunc(_) => todo!(),
        llvm_ir::Instruction::FPExt(_) => todo!(),
        llvm_ir::Instruction::FPToUI(_) => todo!(),
        llvm_ir::Instruction::FPToSI(_) => todo!(),
        llvm_ir::Instruction::UIToFP(_) => todo!(),
        llvm_ir::Instruction::SIToFP(_) => todo!(),
        llvm_ir::Instruction::PtrToInt(_) => todo!(),
        llvm_ir::Instruction::IntToPtr(_) => todo!(),
        llvm_ir::Instruction::BitCast(_) => todo!(),
        llvm_ir::Instruction::AddrSpaceCast(_) => todo!(),
        llvm_ir::Instruction::ICmp(_) => todo!(),
        llvm_ir::Instruction::FCmp(_) => todo!(),
        llvm_ir::Instruction::Phi(_) => todo!(),
        llvm_ir::Instruction::Select(_) => todo!(),
        llvm_ir::Instruction::Freeze(_) => todo!(),
        llvm_ir::Instruction::Call(_) => todo!(),
        llvm_ir::Instruction::VAArg(_) => todo!(),
        llvm_ir::Instruction::LandingPad(_) => todo!(),
        llvm_ir::Instruction::CatchPad(_) => todo!(),
        llvm_ir::Instruction::CleanupPad(_) => todo!(),
    }
}

impl Operand {
    #[inline]
    #[allow(unused)]
    pub fn try_from_llvm(
        operand: &llvm_ir::Operand,
        stack_allocator: &mut StackAllocator,
        stack_slots: &mut HashMap<Name, StackSlot>,
    ) -> Result<Self, BackendError> {
        Ok(match operand {
            llvm_ir::Operand::LocalOperand { name, ty } => {
                let ss = stack_slots
                    .get(name)
                    .ok_or(BackendError::GenFromLlvmError(format!(
                        "stack slot {} not found",
                        name
                    )))?;
                (ss.start() as i64).into()
            }
            llvm_ir::Operand::ConstantOperand(c) => match c.as_ref() {
                Constant::Int { bits: _bits, value } => Operand::Imm((*value as i64).into()),
                Constant::Float(f) => match f {
                    llvm_ir::constant::Float::Single(f) => Operand::Fmm((*f as f64).into()),
                    llvm_ir::constant::Float::Double(_) => {
                        unimplemented!("double float");
                    }
                    _ => {
                        unreachable!();
                    }
                },
                _ => todo!(),
            },
            llvm_ir::Operand::MetadataOperand => todo!(),
        })
    }
}

fn build_bb(
    bb: &llvm_ir::BasicBlock,
    stack_allocator: &mut StackAllocator,
    stack_slots: &mut HashMap<Name, StackSlot>,
) -> Result<Block, BackendError> {
    let mut m_bb = Block::new(bb.name.to_string());
    for inst in &bb.instrs {
        let gen_insts = build_instruction(inst, stack_allocator, stack_slots)?;
        m_bb.extend_insts(gen_insts);
    }
    let gen_insts = build_term_inst(&bb.term)?;
    m_bb.extend_insts(gen_insts);
    Ok(m_bb)
}

/// alloca instruction only instruct allocating memory on stack,not generate one-one instruction
fn build_alloca_inst(
    alloca: &llvm_ir::instruction::Alloca,
    stack_allocator: &mut StackAllocator,
    stack_slots: &mut HashMap<Name, StackSlot>,
) -> Result<Vec<Inst>, BackendError> {
    let name = alloca.dest.clone();
    let ty = alloca.allocated_type.clone();
    let bits = match ty.as_ref() {
        llvm_ir::Type::IntegerType { bits } => *bits,
        _ => todo!(),
    };
    let ss = stack_allocator.alloc(bits as usize);
    stack_slots.insert(name.clone(), ss.clone());
    Ok(vec![])
}

#[allow(unused)]
fn build_store_inst(
    store: &llvm_ir::instruction::Store,
    stack_allocator: &mut StackAllocator,
    stack_slots: &mut HashMap<Name, StackSlot>,
) -> Result<Vec<Inst>, BackendError> {
    let address = &store.address;
    let val = &store.value;
    let address = Operand::try_from_llvm(address, stack_allocator, stack_slots)?;
    let val: Operand = Operand::try_from_llvm(val, stack_allocator, stack_slots)?;
    let mut ret: Vec<Inst> = Vec::new();
    match val {
        Operand::Imm(imm) => {
            let dst = Reg::gen_virtual_usual_reg();
            let li = AddInst::new(dst.clone().into(), REG_ZERO.into(), imm.into());
            let sd = SdInst::new(dst, address.try_into()?, REG_SP);
            ret.push(li.into());
            ret.push(sd.into());
        }
        Operand::Fmm(_) => {
            return Err(BackendError::GenFromLlvmError(
                "store instruction with float value".to_string(),
            ))
        }
        _ => (),
    }
    Ok(ret)
}

fn build_term_inst(term: &llvm_ir::Terminator) -> Result<Vec<Inst>, BackendError> {
    let mut ret_insts: Vec<Inst> = Vec::new();
    match term {
        llvm_ir::Terminator::Ret(r) => {
            if let Some(op) = &r.return_operand {
                match op {
                    llvm_ir::Operand::LocalOperand { name: _, ty: _ } => todo!(),
                    llvm_ir::Operand::ConstantOperand(c) => match c.as_ref() {
                        Constant::Int { bits: _, value } => {
                            let imm = (*value as i64).into();
                            let addi = AddInst::new(REG_A0.into(), REG_ZERO.into(), imm);
                            ret_insts.push(addi.into());
                            ret_insts.push(Inst::Ret);
                        }
                        Constant::Float(_) => todo!(),
                        _ => todo!(),
                    },
                    llvm_ir::Operand::MetadataOperand => todo!(),
                }
            } else {
                unimplemented!();
            }
        }
        _ => todo!(),
    }
    Ok(ret_insts)
}
