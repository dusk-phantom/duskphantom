use super::*;

#[allow(unused)]
pub fn optimize(program: &mut prog::Program) {
    // 不调整块顺序的优化
    for m in program.modules.iter_mut() {
        for f in m.funcs.iter_mut() {
            // inst combine? 匹配一些模式,将多条指令合并成一条
            // mul and div to shift
            // inst scheduling
            // register allocation
            // processing caller-save and callee-save
            // processing stack frame's opening and closing
            // block reordering
        }
    }
}
