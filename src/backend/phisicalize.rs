use super::*;

// turn virtual backend module to phisic backend module
#[allow(unused)]
pub fn phisicalize(program: &mut Program) {
    for module in program.modules.iter_mut() {
        for func in module.funcs.iter_mut() {
            let mut stack_size = 0;
            // count stack size: 统计栈大小,首先遍历每个块每条指令,统计中函数调用的最大栈大小
            // insert prologue: 在函数开头插入: 开栈、保存callee save寄存器
            // insert epilogue: 在函数返回块插入: 关栈、恢复callee save寄存器
            // insert caller save: 在调用前插入: 保存caller save寄存器
            // 检测是否存在长地址访问问题,如果没有,则不保留长地址访问需要的寄存器
            // alloc reg,insert spill: 分配寄存器,插入spill指令,在需要时溢出寄存器值到栈上
            // 虚拟栈地址物理化,根据使用频率和代码结构调整分配的地址
            // 处理长地址访问问题: 使用之前保留的寄存器进行二次计算使用的内存地址
        }
    }
}
