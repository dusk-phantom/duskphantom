use super::*;

/// 函数
/// 函数主要结构为函数名、入口基本块、返回值、参数列表
/// 其中，入口基本块为entry，出口基本块为exit，二者类型为Option<BBPtr>，若为库函数，则均为None
/// 函数内部的数据流结构为基本块组成的有向图，其中该图只有一个入口和一个出口，分别为entry和exit
/// 只有一个入口和一个出口有利于数据流分析
pub struct Function {
    name: String,
    entry: Option<BBPtr>,
    exit: Option<BBPtr>,
    return_type: ValueType,
    // TODO: 参数列表
}

impl Function {
    /// 构造一个新的函数
    pub fn new(name: String, return_type: ValueType) -> Self {
        Self {
            name,
            entry: None,
            exit: None,
            return_type,
        }
    }

    /// 获取函数名
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// 设置函数名
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// 获取返回值类型
    pub fn get_return_type(&self) -> ValueType {
        self.return_type
    }

    /// 设置返回值类型
    pub fn set_return_type(&mut self, return_type: ValueType) {
        self.return_type = return_type;
    }

    /// 获取函数入口，若为库函数，则返回None
    pub fn get_entry(&self) -> Option<BBPtr> {
        self.entry
    }

    /// 设置函数入口
    pub fn set_entry(&mut self, bb: BBPtr) {
        self.entry = Some(bb);
    }

    /// 获取函数出口，若为库函数，则返回None
    pub fn get_exit(&self) -> Option<BBPtr> {
        self.exit
    }

    /// 设置函数出口
    pub fn set_exit(&mut self, bb: BBPtr) {
        self.exit = Some(bb);
    }

    /// 检查是否为库函数
    pub fn is_lib(&self) -> bool {
        self.entry.is_none()
    }
}
