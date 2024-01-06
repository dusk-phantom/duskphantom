// 指定该模块的所有内容条件编译
#[cfg(feature = "clang_frontend")]
use clang::*;

pub struct Program {
    tu: TranslationUnit<'_>,
    llvm: LLVM,
}

impl Program {
    pub fn parse(file: &str) -> Self {
        let clang = Clang::new().unwrap();
        let index = Index::new(&clang, false, false);
        let tu = index.parser("f.c").parse().unwrap();
        TranslationUnit::set_emit_option(&tu, Emit::LLVM);
        let llvm = tu.emit().unwrap();
        Self { tu, llvm }
    }
}
