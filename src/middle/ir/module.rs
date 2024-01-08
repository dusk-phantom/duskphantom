use super::*;

/// 中间代码形式，由Moudle组织
/// 重点关注的是function、global_variables
/// functions是函数集合，需要保证下标为0时为main函数，其余的位置可以随意安排
/// global_variables是全局变量集合，使用HashMap是为了便于根据名字查找，且并不要求顺序
/// Index可以抽象为对应对象的下标，通过相应的函数即可获得
pub struct Module {
    context_arena: Pin<Box<ContextArena>>,

    global_variables: HashMap<String, Index>,
    functions: Vec<(String, Index)>,
}

impl Module {
    /// 构造一个空的Module
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            global_variables: HashMap::new(),
            context_arena: Box::pin(ContextArena::new()),
        }
    }

    /// 构造一个新的函数
    pub fn new_function(&mut self, name: String) -> Index {
        let id = self.context_arena.new_function(&name);
        self.functions.push((name, id));
        id
    }
}
