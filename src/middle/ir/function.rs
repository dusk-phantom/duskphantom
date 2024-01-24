use self::program_mem_pool::ProgramMemPool;

use super::*;

pub type FunPtr = ObjPtr<Function>;

/// 函数
pub struct Function {
    /// mem_pool
    pub mem_pool: ObjPtr<ProgramMemPool>,

    /// 函数名
    pub name: String,

    /// 函数入口，若为库函数，则为None
    pub entry: Option<BBPtr>,

    /// 函数出口，若为库函数，则为None
    pub exit: Option<BBPtr>,

    /// 返回值类型
    pub return_type: ValueType,

    /// 函数参数，存放在一个基本块中
    /// 便于统一指令的运算
    pub params: BBPtr,
}

impl<'func> Function {
    /// 构造一个空函数
    pub fn new(name: String, return_type: ValueType, mem_pool: ObjPtr<ProgramMemPool>) -> Self {
        let params = mem_pool.alloc_basic_block(BasicBlock::new("params".to_string()));
        // let mem_pool = ObjPtr::new(mem_pool.asre);
        Self {
            mem_pool,
            name,
            entry: None,
            exit: None,
            return_type,
            params: params,
        }
    }

    /// 检查是否为库函数
    pub fn is_lib(&self) -> bool {
        self.entry.is_none()
    }
}
