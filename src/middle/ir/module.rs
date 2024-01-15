use super::*;

/// 中间代码形式，由Moudle组织
pub struct Module {
    /// 全局变量集合，存放于基本块中，便于操作
    global_variables: BBPtr,

    /// functions是函数集合，需要保证下标为0时为main函数，其余的位置可以随意安排
    functions: Vec<FunPtr>,
}

impl Module {
    /// 构造一个空的Module
    pub fn new() -> Self {
        // 初始化内存池
        mem_pool::pool_init();
        Self {
            functions: Vec::new(),
            global_variables: mem_pool::alloc_basic_block(BasicBlock::new("global".to_string())),
        }
    }

    /// 根据索引获取函数
    pub fn get_function(&self, index: usize) -> FunPtr {
        self.functions[index]
    }

    /// 调用vec的insert方法插入一个函数
    pub fn insert_function(&mut self, index: usize, function: FunPtr) {
        self.functions.insert(index, function);
    }

    /// 调用vec的push方法在末尾插入一个函数
    pub fn push_function(&mut self, function: FunPtr) {
        self.functions.push(function);
    }

    /// 获得全局变量所在的基本块
    pub fn get_global_variables(&self) -> BBPtr {
        self.global_variables
    }

    /// 插入全局变量
    pub fn insert_global_variable(&mut self, global_variable: InstPtr) {
        self.global_variables.push_back(global_variable);
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        // 释放内存池
        mem_pool::pool_clear();
    }
}
