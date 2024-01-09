use super::*;
pub struct Function {
    name: String,
}

impl Function {
    /// 构造一个新的函数
    pub fn new(name: String) -> Self {
        Self { name }
    }

    /// 获取函数名
    pub fn get_name(&self) -> &String {
        &self.name
    }
}
