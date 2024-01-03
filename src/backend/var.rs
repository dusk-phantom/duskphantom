use super::*;

#[derive(Clone)]
pub enum Var {
    Prim(PrimVar),
    Str(Str),
    Arr(ArrVar),
}

#[derive(Clone)]
pub enum PrimVar {
    Int(Int),
    Float(Float),
}
#[derive(Clone)]
pub struct Int {
    pub name: String,
    pub init: Option<i64>,
}
#[derive(Clone)]
pub struct Str {
    pub name: String,
    pub init: Option<String>,
}
impl Str {
    fn gen_asm(&self) -> String {
        String::new()
    }
}
#[derive(Clone)]
pub struct Float {
    pub name: String,
    pub init: Option<f64>,
}
#[derive(Clone)]
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
        match self {
            Var::Prim(prim) => prim.gen_asm(),
            Var::Str(str) => str.gen_asm(),
            Var::Arr(arr) => arr.gen_asm(),
        }
    }
}
