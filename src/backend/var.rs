use super::*;
pub enum Var {
    Prim(PrimVar),
    Arr(ArrVar),
}
pub enum PrimVar {
    Int(Int),
    Float(Float),
}
pub struct Int {
    pub name: String,
    pub init: i64,
    pub is_const: bool,
}
pub struct Float {
    pub name: String,
    pub init: f64,
    pub is_const: bool,
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
