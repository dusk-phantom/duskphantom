use super::*;

/// 用于分配函数的内存池
static mut FUN_POOL: OnceLock<ObjPool<Function>> = OnceLock::new();

/// 用于分配基本块的内存池
static mut BB_POOL: OnceLock<ObjPool<BasicBlock>> = OnceLock::new();

/// 用于分配指令的内存池
static mut INST_POOL: OnceLock<ObjPool<Box<dyn Instruction>>> = OnceLock::new();

/// 初始化内存池
/// 请确保初始化后再使用后续函数
pub fn pool_init() {
    unsafe {
        let _ = FUN_POOL.set(ObjPool::new());
        let _ = BB_POOL.set(ObjPool::new());
        let _ = INST_POOL.set(ObjPool::new());
    }
}

/// 清空内存池，使其恢复到未初始化的状态
/// 在使用此函数后再使用其分配的空间的指针为未定义行为
pub fn pool_clear() {
    unsafe {
        FUN_POOL.take();
        BB_POOL.take();
        INST_POOL.take();
    }
}

/// 分配一个函数
pub fn alloc_function(func: Function) -> FunPtr {
    unsafe {
        let pool = FUN_POOL.get_mut().unwrap();
        pool.alloc(func)
    }
}

/// 分配一个基本块
pub fn alloc_basic_block(bb: BasicBlock) -> BBPtr {
    unsafe {
        let pool = BB_POOL.get_mut().unwrap();
        pool.alloc(bb)
    }
}

/// 分配一个指令
pub fn alloc_instruction(inst: Box<dyn Instruction>) -> InstPtr {
    unsafe {
        let pool = INST_POOL.get_mut().unwrap();
        pool.alloc(inst)
    }
}
