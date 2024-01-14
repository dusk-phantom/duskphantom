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

    /// 函数参数，存放在一个基本块中
    /// 便于统一指令的运算
    params: BBPtr,
}

impl<'func> Function {
    /// 构造一个空函数
    pub fn new(name: String, return_type: ValueType) -> Self {
        Self {
            name,
            entry: None,
            exit: None,
            return_type,
            params: mem_pool::alloc_basic_block(BasicBlock::new("params".to_string())),
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

    /// 获取函数参数所在基本块
    pub fn get_params(&self) -> BBPtr {
        self.params
    }

    /// 在参数块尾部插入一个参数
    pub fn push_param(&mut self, param: InstPtr) {
        todo!()
    }

    /// 检查是否为库函数
    pub fn is_lib(&self) -> bool {
        self.entry.is_none()
    }
}
