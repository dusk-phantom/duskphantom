// 一个module 是一个 基础独立编译单元, 或者理解成c的一个 独立代码文件

use super::*;
use crate::config::CONFIG;
pub struct Module {
    // module name
    pub name: String,
    // global var ,including primtype var and arr var
    pub global: Vec<var::Var>,
    // all funcs
    pub funcs: Vec<func::Func>,
    // entry func name
    pub entry: Option<String>,
}

impl Module {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn entry(&self) -> Option<&func::Func> {
        if let Some(entry) = self.entry.as_ref() {
            for func in self.funcs.iter() {
                if func.name() == entry.as_str() {
                    return Some(func);
                }
            }
        }
        None
    }
    pub fn gen_asm(&self) -> String {
        //
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(CONFIG.num_parallel_for_global_gen_asm)
            .build()
            .unwrap();
        let global = thread_pool.install(|| {
            self.global
                .par_iter()
                .map(|v| v.gen_asm())
                .collect::<Vec<String>>()
                .join("\n")
        });
        let mut funcs: Vec<&func::Func> = self.funcs.iter().collect();
        funcs.sort_by_cached_key(|f| f.name());
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(CONFIG.num_parallel_for_func_gen_asm)
            .build()
            .unwrap();
        let funcs = thread_pool.install(|| {
            funcs
                .par_iter()
                .map(|f| f.gen_asm())
                .collect::<Vec<String>>()
                .join("\n")
        });
        gen_asm::GenTool::gen_prog("test.c", global.as_str(), funcs.as_str())
    }
}
