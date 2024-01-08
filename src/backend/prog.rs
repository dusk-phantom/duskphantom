use super::*;
pub struct Program {
    // global var ,including primtype var and arr var
    pub global: Vec<var::Var>,
    // all funcs
    pub funcs: Vec<func::Func>,
    // optional entry func
    pub entry: Option<String>,
}

impl Program {
    pub fn has_entry(&self) -> bool {
        self.entry.is_some()
    }
    pub fn gen_asm(&self) -> String {
        let mut funcs: Vec<&func::Func> = self.funcs.iter().collect();
        funcs.sort_by_cached_key(|f| f.name());
        let funcs = funcs
            .par_iter()
            .map(|f| f.gen_asm())
            .collect::<Vec<String>>()
            .join("\n");
        let global = self
            .global
            .par_iter()
            .map(|v| v.gen_asm())
            .collect::<Vec<String>>()
            .join("\n");
        asm::GenTool::gen_prog("test.c", global.as_str(), funcs.as_str())
    }
}
