# phisicalize是从后端ir到riscv汇编转换的过程(主要用于不开启优化的情况下验证功能),分为多个阶段

- handle_illegal_inst: 处理非法指令,将ir中的非法指令转换为riscv汇编中的合法指令

- phisicalize_reg: 使用t0,t1,t2寄存器,将ir中的寄存器转换为riscv汇编中的寄存器

- ~~handle_callee_save: 处理callee save寄存器,将保存和恢复callee save寄存器的代码插入到每个函数的entry前和exit结尾的terminator前,并且使用stack_slot_allocator为callee save寄存器分配栈空间~~ (发现不需要这个过程)

- ~~handle_caller_save: 处理caller save寄存器,将保存和恢复caller save寄存器的代码插入到call指令前和call指令后,并且使用stack_slot_allocator为caller save寄存器分配栈空间~~ (发现不需要这个过程)

- handle_ra. 在函数的开头和退出前插入保存和恢复ra的代码

- handle_stack: 处理栈的开闭和栈帧指针s0的保存和恢复,在这一步我们终于能够确定为这个函数分配的栈空间的大小,于是以16字节对齐的方式为这个函数分配栈空间,然后在函数的开头和退出前插入保存和恢复sp的代码(开栈操作),以及插入保存和恢复s0的代码,我们会在函数开头开栈(更新sp)后,将sp的值赋给s0,在函数退出前,将s0的值赋给sp(闭栈操作)

- handle_mem: 处理内存操作,将ir中的内存操作转换为riscv汇编中的内存操作。举例来说,对于一个LoadInst,我们会转换成对应的ld指令或者lw指令

- handle_offset_overflows: 处理offset溢出,在这一步我们会检查所有的offset,如果offset超过了12位,我们会将这个offset拆分成两个offset,并且使用t3寄存器保存中间值进行计算
