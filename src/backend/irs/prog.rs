use super::*;

// 一个program是一个程序, 可能由多个 module组成
pub struct Program {
    /// optional entry module name, to specify if this program is a library or executable
    pub entry: Option<String>,
    pub modules: Vec<module::Module>,
}

impl Program {
    pub fn entry(&self) -> Option<&module::Module> {
        if let Some(entry) = self.entry.as_ref() {
            for module in self.modules.iter() {
                if module.name() == entry.as_str() {
                    return Some(module);
                }
            }
        }
        None
    }
    pub fn gen_asm(&self) -> String {
        // Note: only consider single module program now
        let mut asm = String::with_capacity(1024 * 1024);
        for module in self.modules.iter() {
            asm.push_str(module.gen_asm().as_str());
        }
        asm
    }
}
