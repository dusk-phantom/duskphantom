#[macro_export]
macro_rules! impl_three_op_inst {
    ($ty_name:ident,$inst_name:expr) => {
        #[derive(Clone, Debug)]
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

                if let Operand::Reg(r) = self.dst() {
                    if r.is_usual() {
                        if matches!(self.rhs(), Operand::Imm(_)) {
                            format!("{}i {},{},{}", $inst_name, dst, lhs, rhs)
                        } else {
                            format!("{} {},{},{}", $inst_name, dst, lhs, rhs)
                        }
                    } else {
                        format!("f{}.s {},{},{}", $inst_name, dst, lhs, rhs)
                    }
                } else {
                    format!("{} {},{},{}", $inst_name, dst, lhs, rhs)
                }
            }
        }
        impl RegDefs for $ty_name {
            fn defs(&self) -> Vec<&Reg> {
                if let Operand::Reg(reg) = self.dst() {
                    vec![reg]
                } else {
                    vec![]
                }
            }
        }
        impl RegUses for $ty_name {
            fn uses(&self) -> Vec<&Reg> {
                let mut regs = Vec::with_capacity(2);
                if let Operand::Reg(r1) = self.lhs() {
                    regs.push(r1);
                    if let Operand::Reg(r2) = self.rhs() {
                        if r2 != r1 {
                            regs.push(r2);
                        }
                    }
                } else if let Operand::Reg(reg) = self.rhs() {
                    regs.push(reg);
                }
                regs
            }
        }
        impl RegReplace for $ty_name {
            fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
                if let Operand::Reg(r1) = self.lhs_mut() {
                    if *r1 == from {
                        *r1 = to;
                    }
                }
                if let Operand::Reg(r2) = self.rhs_mut() {
                    if *r2 == from {
                        *r2 = to;
                    }
                }
                Ok(())
            }
            fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
                if let Operand::Reg(r) = self.dst_mut() {
                    if *r == from {
                        *r = to;
                    }
                }
                Ok(())
            }
        }
    };
}
#[macro_export]
macro_rules! impl_three_op_inst_with_dstmem {
    ($ty_name:ident,$inst_name:expr) => {
        #[derive(Clone, Debug)]
        pub struct $ty_name(Operand, Operand, Operand, bool);
        impl $ty_name {
            pub fn new(dst: Operand, lhs: Operand, rhs: Operand) -> Self {
                Self(dst, lhs, rhs, false)
            }

            pub fn with_8byte(mut self) -> Self {
                self.3 = true;
                self
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

                if let Operand::Reg(r) = self.dst() {
                    if r.is_usual() {
                        let with_8byte = self.3;
                        if matches!(self.rhs(), Operand::Imm(_)) {
                            if with_8byte {
                                format!("{}i {},{},{}", $inst_name, dst, lhs, rhs)
                            } else {
                                format!("{}iw {},{},{}", $inst_name, dst, lhs, rhs)
                            }
                        } else {
                            if with_8byte {
                                format!("{} {},{},{}", $inst_name, dst, lhs, rhs)
                            } else {
                                format!("{}w {},{},{}", $inst_name, dst, lhs, rhs)
                            }
                        }
                    } else {
                        format!("f{}.s {},{},{}", $inst_name, dst, lhs, rhs)
                    }
                } else {
                    format!("{} {},{},{}", $inst_name, dst, lhs, rhs)
                }
            }
        }
        impl RegDefs for $ty_name {
            fn defs(&self) -> Vec<&Reg> {
                if let Operand::Reg(reg) = self.dst() {
                    vec![reg]
                } else {
                    vec![]
                }
            }
        }
        impl RegUses for $ty_name {
            fn uses(&self) -> Vec<&Reg> {
                let mut regs = Vec::with_capacity(2);
                if let Operand::Reg(r1) = self.lhs() {
                    regs.push(r1);
                    if let Operand::Reg(r2) = self.rhs() {
                        if r2 != r1 {
                            regs.push(r2);
                        }
                    }
                } else if let Operand::Reg(reg) = self.rhs() {
                    regs.push(reg);
                }
                regs
            }
        }
        impl RegReplace for $ty_name {
            fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
                if let Operand::Reg(r1) = self.lhs_mut() {
                    if *r1 == from {
                        *r1 = to;
                    }
                }
                if let Operand::Reg(r2) = self.rhs_mut() {
                    if *r2 == from {
                        *r2 = to;
                    }
                }
                Ok(())
            }
            fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
                if let Operand::Reg(r) = self.dst_mut() {
                    if *r == from {
                        *r = to;
                    }
                }
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_branch_inst {
    ($ty_name:ident,$inst_name:expr) => {
        #[derive(Clone, Debug)]
        pub struct $ty_name(Reg, Reg, Label);
        impl $ty_name {
            pub fn new(dst: Reg, lhs: Reg, rhs: Label) -> Self {
                Self(dst, lhs, rhs)
            }
            #[inline]
            pub fn lhs(&self) -> &Reg {
                &self.0
            }
            #[inline]
            pub fn rhs(&self) -> &Reg {
                &self.1
            }
            #[inline]
            pub fn lhs_mut(&mut self) -> &mut Reg {
                &mut self.0
            }
            #[inline]
            pub fn rhs_mut(&mut self) -> &mut Reg {
                &mut self.1
            }
            #[inline]
            pub fn label(&self) -> &Label {
                &self.2
            }
            #[inline]
            pub fn label_mut(&mut self) -> &mut Label {
                &mut self.2
            }
            #[inline]
            pub fn gen_asm(&self) -> String {
                let lhs = self.lhs().gen_asm();
                let rhs = self.rhs().gen_asm();
                let label = self.label().gen_asm();
                format!("{} {},{},{}", $inst_name, lhs, rhs, label)
            }
        }
        impl RegDefs for $ty_name {
            fn defs(&self) -> Vec<&Reg> {
                vec![]
            }
        }
        impl RegUses for $ty_name {
            fn uses(&self) -> Vec<&Reg> {
                let (l, r) = (self.lhs(), self.rhs());
                if l == r {
                    vec![l]
                } else {
                    vec![l, r]
                }
            }
        }

        impl RegReplace for $ty_name {
            fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
                if *self.lhs_mut() == from {
                    *self.lhs_mut() = to;
                }
                if *self.rhs_mut() == from {
                    *self.rhs_mut() = to;
                }
                Ok(())
            }
        }

        impl ToBB for $ty_name {
            fn to_bb(&self) -> Result<&str> {
                Ok(&self.2.as_str())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_two_op_inst {
    ($ty_name:ident,$inst_name:expr) => {
        #[derive(Clone, Debug)]
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
                if let Operand::Reg(r) = self.dst() {
                    if r.is_usual() {
                        format!("{} {},{}", $inst_name, dst, src)
                    } else {
                        format!("f{}.s {},{}", $inst_name, dst, src)
                    }
                } else {
                    unreachable!()
                }
            }
        }
        impl RegDefs for $ty_name {
            fn defs(&self) -> Vec<&Reg> {
                if let Operand::Reg(reg) = self.dst() {
                    vec![reg]
                } else {
                    vec![]
                }
            }
        }
        impl RegUses for $ty_name {
            fn uses(&self) -> Vec<&Reg> {
                if let Operand::Reg(reg) = self.src() {
                    vec![reg]
                } else {
                    vec![]
                }
            }
        }
        impl RegReplace for $ty_name {
            fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
                if let Operand::Reg(reg) = self.src_mut() {
                    if *reg == from {
                        *reg = to;
                    }
                }
                Ok(())
            }
            fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
                if let Operand::Reg(reg) = self.dst_mut() {
                    if *reg == from {
                        *reg = to;
                    }
                }
                Ok(())
            }
        }
    };
}

#[macro_export]
/// create a new instruction type with two operands for conversion inst like fcvt.s.w and fcvt.w.s
macro_rules! impl_conversion_inst {
    ($ty_name:ident,$inst_name:expr) => {
        #[derive(Clone, Debug)]
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
                format!("{} {},{}", $inst_name, dst, src)
            }
        }
        impl RegDefs for $ty_name {
            fn defs(&self) -> Vec<&Reg> {
                if let Operand::Reg(reg) = self.dst() {
                    vec![reg]
                } else {
                    vec![]
                }
            }
        }
        impl RegUses for $ty_name {
            fn uses(&self) -> Vec<&Reg> {
                if let Operand::Reg(reg) = self.src() {
                    vec![reg]
                } else {
                    vec![]
                }
            }
        }
        impl RegReplace for $ty_name {
            fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
                if let Operand::Reg(reg) = self.src_mut() {
                    if *reg == from {
                        *reg = to;
                    }
                }
                Ok(())
            }
            fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
                if let Operand::Reg(reg) = self.dst_mut() {
                    if *reg == from {
                        *reg = to;
                    }
                }
                Ok(())
            }
        }
    };
    ($ty_name:ident,$inst_name:expr,$inst_suffix:expr) => {
        #[derive(Clone, Debug)]
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
                format!("{} {},{},{}", $inst_name, dst, src, $inst_suffix)
            }
        }
        impl RegDefs for $ty_name {
            fn defs(&self) -> Vec<&Reg> {
                if let Operand::Reg(reg) = self.dst() {
                    vec![reg]
                } else {
                    vec![]
                }
            }
        }
        impl RegUses for $ty_name {
            fn uses(&self) -> Vec<&Reg> {
                if let Operand::Reg(reg) = self.src() {
                    vec![reg]
                } else {
                    vec![]
                }
            }
        }
        impl RegReplace for $ty_name {
            fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
                if let Operand::Reg(reg) = self.src_mut() {
                    if *reg == from {
                        *reg = to;
                    }
                }
                Ok(())
            }
            fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
                if let Operand::Reg(reg) = self.dst_mut() {
                    if *reg == from {
                        *reg = to;
                    }
                }
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_mem_inst {
    ($ty_name:ident,$inst_name:expr) => {
        #[derive(Clone, Debug)]
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
                if self.dst().is_usual() {
                    format!("{} {},{}({})", $inst_name, dst, offset, base)
                } else {
                    format!("f{} {},{}({})", $inst_name, dst, offset, base)
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_unary_inst {
    ($ty_name:ident,$inst_name:expr) => {
        #[derive(Clone, Debug)]
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

        impl RegReplace for $ty_name {
            fn replace_def(&mut self, from: Reg, to: Reg) -> Result<()> {
                if let Operand::Reg(reg) = self.dst_mut() {
                    if *reg == from {
                        *reg = to;
                    }
                }
                Ok(())
            }
            fn replace_use(&mut self, from: Reg, to: Reg) -> Result<()> {
                if let Operand::Reg(reg) = self.dst_mut() {
                    if *reg == from {
                        *reg = to;
                    }
                }
                Ok(())
            }
        }
    };
}

#[macro_export]
/// Implement the conversion between the instruction type and the enum variant.
macro_rules! impl_inst_convert {
    ($inst_type:ident,$enumerator:ident) => {
        impl From<$inst_type> for Inst {
            fn from(value: $inst_type) -> Inst {
                Inst::$enumerator(value)
            }
        }
        impl TryFrom<Inst> for $inst_type {
            type Error = Inst;
            fn try_from(value: Inst) -> Result<$inst_type, Inst> {
                match value {
                    Inst::$enumerator(inst) => Ok(inst),
                    _ => Err(value),
                }
            }
        }
    };
}
