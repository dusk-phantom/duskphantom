use super::*;
pub struct Function {
    name: String,
    context: ObjPtr<ContextArena>,
}

impl Function {
    /// 构造一个新的函数
    pub fn new(name: String, context: &ContextArena) -> Self {
        Self {
            name,
            context: ObjPtr::new(context),
        }
    }

    /// 获取函数名
    pub fn get_name(&self) -> &String {
        &self.name
    }
}
