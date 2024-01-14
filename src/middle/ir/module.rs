use super::*;

/// 中间代码形式，由Moudle组织
/// 重点关注的是function、global_variables
/// functions是函数集合，需要保证下标为0时为main函数，其余的位置可以随意安排
/// global_variables是全局变量集合，使用HashMap是为了便于根据名字查找，且并不要求顺序
/// Index可以抽象为对应对象的下标，通过相应的函数即可获得
pub struct Module {
    /// 全局变量集合，存放于基本块中，便于操作
    global_variables: BBPtr,
    functions: Vec<FunPtr>,
}

impl Module {
    /// 构造一个空的Module
    pub fn new() -> Self {
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
