use std::collections::HashMap;

use anyhow::Result;

use crate::{
    backend::Operand,
    middle::{
        ir::{BBPtr, FunPtr, InstPtr, ParaPtr},
        Program,
    },
};

#[allow(unused)]
pub fn inline_func(func: FunPtr, program: &mut Program) -> Result<()> {
    Ok(())
}

#[allow(unused)]
pub fn inline_no_succ(func: FunPtr, program: &mut Program) -> Result<()> {
    for bb in func.dfs_iter() {}
    Ok(())
}

#[allow(unused)]
pub fn inline_no_self_call(func: FunPtr, program: &mut Program) -> Result<()> {
    Ok(())
}

#[allow(unused)]
pub fn inline_call(block: BBPtr, callee: InstPtr, program: &mut Program) -> Result<()> {
    Ok(())
}

#[allow(unused)]
pub fn mirror_func(
    src: BBPtr,
    dst: BBPtr,
    arg_map: HashMap<ParaPtr, Operand>,
    program: &mut Program,
) -> Result<()> {
    // Copy all instructions in src to dst
    for inst in src.iter() {}
    Ok(())
}
