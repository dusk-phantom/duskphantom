use super::*;
pub enum Var {
    Prim(PrimVar),
    Arr(ArrVar),
}
pub enum PrimVar {
    Int(i64),
    Float(f64),
    Bool(bool),
}
pub struct ArrVar {
    pub name: String,
    pub size: usize,
    pub init: Vec<PrimVar>,
}
impl ArrVar {
    pub fn gen_asm(&self) -> String {
        // TODO
        String::new()
    }
}
impl PrimVar {
    pub fn gen_asm(&self) -> String {
        // TODO
        String::new()
    }
}

impl Var {
    pub fn gen_asm(&self) -> String {
        // TODO
        String::new()
    }
}
