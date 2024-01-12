use super::*;

pub type FunPtr = ObjPtr<Function>;

/// 函数
pub struct Function {
    /// 函数名
    name: String,

    /// 函数入口，若为库函数，则为None
    entry: Option<BBPtr>,

    /// 函数出口，若为库函数，则为None
    exit: Option<BBPtr>,

    /// 返回值类型
    return_type: ValueType,

    /// 函数参数
    params: Vec<InstPtr>,
}

impl<'func> Function {
    /// 构造一个空函数
    pub fn new(name: String, return_type: ValueType) -> Self {
        Self {
            name,
            entry: None,
            exit: None,
            return_type,
            params: Vec::new(),
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

    /// 获取函数参数
    pub fn get_param(&self, index: usize) -> InstPtr {
        self.params[index]
    }
}
