#[macro_export]
macro_rules! impl_three_op_inst {
    ($ty_name:ident,$inst_name:expr) => {
        #[derive(Clone)]
        pub struct $ty_name(Operand, Operand, Operand);
        impl $ty_name {
            pub fn new(dst: Operand, lhs: Operand, rhs: Operand) -> Self {
                Self(dst, lhs, rhs)
            }
            pub fn dst(&self) -> &Operand {
                &self.0
            }
            pub fn lhs(&self) -> &Operand {
                &self.1
            }
            pub fn rhs(&self) -> &Operand {
                &self.2
            }
            pub fn dst_mut(&mut self) -> &mut Operand {
                &mut self.0
            }
            pub fn lhs_mut(&mut self) -> &mut Operand {
                &mut self.1
            }
            pub fn rhs_mut(&mut self) -> &mut Operand {
                &mut self.2
            }

            pub fn gen_asm(&self) -> String {
                let dst = self.dst().gen_asm();
                let lhs = self.lhs().gen_asm();
                let rhs = self.rhs().gen_asm();
                format!("{} {},{},{}", $inst_name, dst, lhs, rhs)
            }
        }
    };
}
#[macro_export]
macro_rules! impl_two_op_inst {
    ($ty_name:ident) => {
        #[derive(Clone)]
        pub struct $ty_name(Operand, Operand);
        impl $ty_name {
            pub fn new(dst: Operand, src: Operand) -> Self {
                Self(dst, src)
            }
            pub fn dst(&self) -> &Operand {
                &self.0
            }
            pub fn src(&self) -> &Operand {
                &self.1
            }
            pub fn dst_mut(&mut self) -> &mut Operand {
                &mut self.0
            }
            pub fn src_mut(&mut self) -> &mut Operand {
                &mut self.1
            }
            pub fn gen_asm(&self) -> String {
                let dst = self.dst().gen_asm();
                let src = self.src().gen_asm();
                format!("{} {},{}", stringify!($ty_name).to_lowercase(), dst, src)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_mem_inst {
    ($ty_name:ident,$inst_name:expr) => {
        #[derive(Clone)]
        pub struct $ty_name(Reg, Imm, Reg);
        impl $ty_name {
            pub fn new(dst: Reg, offset: Imm, base: Reg) -> Self {
                Self(dst, offset, base)
            }
            pub fn dst(&self) -> &Reg {
                &self.0
            }
            pub fn offset(&self) -> &Imm {
                &self.1
            }
            pub fn base(&self) -> &Reg {
                &self.2
            }
            pub fn dst_mut(&mut self) -> &mut Reg {
                &mut self.0
            }
            pub fn offset_mut(&mut self) -> &mut Imm {
                &mut self.1
            }
            pub fn base_mut(&mut self) -> &mut Reg {
                &mut self.2
            }

            pub fn gen_asm(&self) -> String {
                let dst = self.dst().gen_asm();
                let offset = self.offset().gen_asm();
                let base = self.base().gen_asm();
                format!("{} {},{}({})", $inst_name, dst, offset, base)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_unary_inst {
    ($ty_name:ident,$inst_name:expr) => {
        #[derive(Clone)]
        pub struct $ty_name(Operand);
        impl $ty_name {
            pub fn new(dst: Operand) -> Self {
                Self(dst)
            }
            pub fn dst(&self) -> &Operand {
                &self.0
            }
            pub fn dst_mut(&mut self) -> &mut Operand {
                &mut self.0
            }
            pub fn gen_asm(&self) -> String {
                let dst = self.dst().gen_asm();
                format!("{} {}", $inst_name, dst)
            }
        }
    };
}
