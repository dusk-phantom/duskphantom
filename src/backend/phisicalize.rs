use super::*;

// turn virtual backend module to phisic backend module
#[allow(unused)]
pub fn phisicalize(program: &mut Program) -> Result<(), BackendError> {
    for module in program.modules.iter_mut() {
        for func in module.funcs.iter_mut() {
            // count stack size: 统计栈大小,首先遍历每个块每条指令,统计中函数调用的最大栈大小
            let stack_size = func
                .stack_allocator()
                .ok_or(anyhow!("virtual stack allocator not found"))
                .with_context(|| context!())?
                .allocated();
            dbg!(func.name());
            dbg!(stack_size);
            dbg!(Func::in_out_bbs(func));
            dbg!(Func::reg_interfere_graph(func));
            // alloc reg: 调用寄存器分配算法,获得分配结果,其中包括寄存器溢出需要的空间

            // count caller save: 计算保存caller save需要的栈空间 max(caller_save(call[i])),这一步应该在寄存器分配之后进行

            // count callee save: 计算保存callee save需要的栈空间,这一步应该在寄存器分配之后进行

            // count return value: 计算返回值需要的栈空间,返回值的栈空间以栈底作为起点 (略,因为只返回int或者float)

            // process_long_address: 检测是否有长地址访问,如果有,则使用保留寄存器处理长地址访问问题

            // apply reg alloc: 实施寄存器分配结果,插入spill 指令,使用t0-t3三个寄存器处理寄存器溢出问题

            // apply caller save: 插入保存caller save的指令

            // apply callee save: 插入保存callee save的指令
        }
    }
    Ok(())
}
