use super::*;

/// 栈空间操作: 处理ra,sp,s0,的保存和恢复, 以及使用t3寄存器处理访存指令off溢出
pub fn handle_stack(func: &mut Func) -> Result<()> {
    phisicalize::handle_stack(func)?;
    phisicalize::handle_mem(func)?;
    phisicalize::handle_offset_overflows(func)?;
    Ok(())
}
