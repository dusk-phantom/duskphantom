pub enum Inst {
    Add,
    Mv,
    Mul,
    Div,
    Jmp,
    Branch,
    Call,
    Ret,
}

#[derive(Clone, PartialEq, Eq)]
pub enum InstType {
    Add = 0,
    Mv,
    Mul,
    Div,
    Jmp,
    Branch,
    Call,
    Ret,
}

impl Inst {
    pub fn _type(&self) -> InstType {
        match self {
            Inst::Add => InstType::Add,
            Inst::Mv => InstType::Mv,
            Inst::Mul => InstType::Mul,
            Inst::Div => InstType::Div,
            Inst::Jmp => InstType::Jmp,
            Inst::Branch => InstType::Branch,
            Inst::Call => InstType::Call,
            Inst::Ret => InstType::Ret,
        }
    }
}

// unit test
#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    pub fn test_inst_type() {
        let inst = Inst::Add;
        assert_eq!(inst._type() as usize, InstType::Add as usize);
        assert_eq!(inst._type() as usize, 0);
        let inst = Inst::Mv;
        assert_eq!(inst._type() as usize, InstType::Mv as usize);
        assert_eq!(inst._type() as usize, 1);
        let inst = Inst::Mul;
        assert_eq!(inst._type() as usize, InstType::Mul as usize);
        assert_eq!(inst._type() as usize, 2);
        let inst = Inst::Div;
        assert_eq!(inst._type() as usize, InstType::Div as usize);
        assert_eq!(inst._type() as usize, 3);
        let inst = Inst::Jmp;
        assert_eq!(inst._type() as usize, InstType::Jmp as usize);
        assert_eq!(inst._type() as usize, 4);
        let inst = Inst::Branch;
        assert_eq!(inst._type() as usize, InstType::Branch as usize);
        assert_eq!(inst._type() as usize, 5);
        let inst = Inst::Call;
        assert_eq!(inst._type() as usize, InstType::Call as usize);
        assert_eq!(inst._type() as usize, 6);
        let inst = Inst::Ret;
        assert_eq!(inst._type() as usize, InstType::Ret as usize);
        assert_eq!(inst._type() as usize, 7);
    }
}
