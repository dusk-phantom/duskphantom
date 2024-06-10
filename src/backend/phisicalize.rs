use super::*;

// turn virtual backend module to phisic backend module
#[allow(unused)]
pub fn phisicalize(mdl: &mut Program) {
    // insert prologue: 在函数开头插入: 开栈、保存callee save寄存器
    // insert epilogue: 在函数返回块插入: 关栈、恢复callee save寄存器
    // insert caller save: 在调用前插入: 保存caller save寄存器
    // alloc reg,with spill

    // todo!();
}
