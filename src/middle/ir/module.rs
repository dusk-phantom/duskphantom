use super::*;

/// 中间代码形式，由Moudle组织
/// 重点关注的是function、global_variables
/// functions是函数集合，需要保证下标为0时为main函数，其余的位置可以随意安排
/// global_variables是全局变量集合，使用HashMap是为了便于根据名字查找，且并不要求顺序
/// Index可以抽象为对应对象的下标，通过相应的函数即可获得
pub struct Module {
    global_variables: Vec<InstPtr>,
    functions: Vec<FunPtr>,
}

impl Module {
    /// 构造一个空的Module
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            global_variables: Vec::new(),
        }
    }
}
