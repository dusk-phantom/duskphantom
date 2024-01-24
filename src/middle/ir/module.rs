use crate::backend::prog::Program;

use self::program_mem_pool::ProgramMemPool;

use super::*;

/// 中间代码形式，由Moudle组织
pub struct Module {
    /// 全局变量集合，存放于基本块中，便于操作
    pub global_variables: BBPtr,
    pub mem_pool: ObjPtr<ProgramMemPool>,
    /// functions是函数集合，需要保证下标为0时为main函数，其余的位置可以随意安排
    pub functions: Vec<FunPtr>,
}

impl Module {
    /// 构造一个空的Module
    pub fn new(mem_pool: ObjPtr<ProgramMemPool>) -> Self {
        let mut mut_mem_pool: ObjPtr<ProgramMemPool> = mem_pool.clone();
        let mut_mem_pool: &mut ProgramMemPool = mut_mem_pool.as_mut();
        let global_variables =
            mut_mem_pool.alloc_basic_block(BasicBlock::new("global".to_string(), mem_pool));
        Self {
            functions: Vec::new(),
            mem_pool,
            global_variables: global_variables,
        }
    }
}
