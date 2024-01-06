use clang::*;

pub struct Program {
    tu: TranslationUnit<'_>,
}

impl Program {
    pub fn new(file: &str) -> Self {
        let clang = Clang::new().unwrap();
        let index = Index::new(&clang, false, false);
        let tu = index.parser("f.c").parse().unwrap();
        Self { tu }
    }
}
