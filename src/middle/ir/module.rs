use super::*;

/// one module is one file
pub struct Module {
    /// global variables in this module
    pub global_variables: BBPtr,

    /// functions in this module.
    /// Make sure that the first function is `main` function.
    pub functions: Vec<FunPtr>,

    pub mem_pool: ObjPtr<IRBuilder>,
}

impl Module {
    pub fn new(mut mem_pool: ObjPtr<IRBuilder>) -> Self {
        let global_variables = mem_pool.new_basicblock("global".to_string());
        Self {
            functions: Vec::new(),
            mem_pool,
            global_variables,
        }
    }
}
