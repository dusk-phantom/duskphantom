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
        let ret_ty = &f.return_type;
        let mut stack_allocator = StackAllocator::new();
        let mut stack_slots: HashMap<Name, StackSlot> = HashMap::new();
        // dbg!(&ret_ty);
        for bb in &f.basic_blocks {
            // dbg!(bb);
            for inst in &bb.instrs {
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
                        let dest = &alloca.dest;
                        match &alloca.allocated_type.as_ref() {
                            llvm_ir::Type::IntegerType { bits } => {
                                let ss = stack_allocator.alloc(*bits as usize);
                                stack_slots.insert(dest.clone(), ss);
                            }
                            _ => todo!(),
                        };
                    }
                    llvm_ir::Instruction::Load(_) => todo!(),
                    llvm_ir::Instruction::Store(store) => {
                        let address = &store.address;
                        let val = &store.value;
                        let address = match address {
                            llvm_ir::Operand::LocalOperand { name, ty } => {
                                let ss = stack_slots.get(name).unwrap();
                                ss
                            }
                            llvm_ir::Operand::ConstantOperand(_) => todo!(),
                            llvm_ir::Operand::MetadataOperand => todo!(),
                        };
                        // #[allow(unused)]
                        // let val = match val {
                        //     llvm_ir::Operand::LocalOperand { name, ty } => {
                        //         todo!()
                        //     }
                        //     llvm_ir::Operand::ConstantOperand(c) => match c.as_ref() {
                        //         Constant::Int { bits, value } => {
                        //             // crate::backend::Operand::Imm(Imm::from(*value as i32))
                        //         }
                        //         Constant::Float(_) => todo!(),
                        //         _ => todo!(),
                        //     },
                        //     llvm_ir::Operand::MetadataOperand => todo!(),
                        // };

                        dbg!(&store);
                        todo!();
                    }
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
        }
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
