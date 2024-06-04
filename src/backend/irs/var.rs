use gen_asm::{Data, GenTool};

#[allow(unused)]
use super::*;

#[derive(Clone, Debug)]
pub enum Var {
    Prim(PrimVar),
    Str(Str),
    IntArr(ArrVar<u32>),
    FloatArr(ArrVar<f32>),
}

#[derive(Clone, Debug)]
pub enum PrimVar {
    IntVar(IntVar),
    FloatVar(FloatVar),
}
#[derive(Clone, Debug)]
pub struct IntVar {
    pub name: String,
    pub init: Option<i32>,
    pub is_const: bool,
}
#[derive(Clone, Debug)]
pub struct Str {
    pub name: String,
    pub init: Option<String>,
    pub is_const: bool,
}
impl Str {
    fn gen_asm(&self) -> String {
        let init = self.init.as_deref().unwrap_or("");
        GenTool::gen_const_str(&self.name, init)
    }
}
#[derive(Clone, Debug)]
pub struct FloatVar {
    pub name: String,
    pub init: Option<f32>,
    pub is_const: bool,
}
#[derive(Clone, Debug)]
pub struct ArrVar<T: Data> {
    pub name: String,
    pub capacity: usize,
    pub init: Vec<(usize, T)>,
    pub is_const: bool,
}

impl<T: Data> ArrVar<T> {
    pub fn gen_asm(&self) -> String {
        GenTool::gen_array(&self.name, self.capacity, &self.init)
    }
}
impl PrimVar {
    pub fn gen_asm(&self) -> String {
        match self {
            PrimVar::IntVar(var) => var.gen_asm(),
            PrimVar::FloatVar(var) => var.gen_asm(),
        }
    }
}
impl IntVar {
    pub fn gen_asm(&self) -> String {
        GenTool::gen_int(&self.name, self.init.unwrap_or(0))
    }
}
impl FloatVar {
    pub fn gen_asm(&self) -> String {
        GenTool::gen_float(&self.name, self.init.unwrap_or(0.0))
    }
}

impl Var {
    pub fn gen_asm(&self) -> String {
        match self {
            Var::Prim(prim) => prim.gen_asm(),
            Var::Str(str) => str.gen_asm(),
            Var::IntArr(arr) => arr.gen_asm(),
            Var::FloatArr(arr) => arr.gen_asm(),
        }
    }
}
