use super::*;

impl IRBuilder {
    pub fn get_icmp(
        &mut self,
        op: ICmpOp,
        comp_type: ValueType,
        lhs: Operand,
        rhs: Operand,
    ) -> InstPtr {
        let mut inst = self.new_instruction(Box::new(ICmp {
            op,
            comp_type,
            manager: InstManager::new(ValueType::Bool),
        }));
        unsafe {
            inst.get_manager_mut().add_operand(lhs);
            inst.get_manager_mut().add_operand(rhs);
        };
        inst
    }

    pub fn get_fcmp(
        &mut self,
        op: FCmpOp,
        comp_type: ValueType,
        lhs: Operand,
        rhs: Operand,
    ) -> InstPtr {
        let mut inst = self.new_instruction(Box::new(FCmp {
            op,
            comp_type,
            manager: InstManager::new(ValueType::Bool),
        }));
        unsafe {
            inst.get_manager_mut().add_operand(lhs);
            inst.get_manager_mut().add_operand(rhs);
        };
        inst
    }

    pub fn get_phi(&mut self, ty: ValueType, incoming_values: Vec<(Operand, BBPtr)>) -> InstPtr {
        let mut inst = self.new_instruction(Box::new(Phi {
            incoming_values: incoming_values.clone(),
            manager: InstManager::new(ty),
        }));
        for (val, _) in &incoming_values {
            unsafe {
                inst.get_manager_mut().add_operand(val.clone());
            }
        }
        inst
    }

    pub fn get_call(&mut self, func: FunPtr, args: Vec<Operand>) -> InstPtr {
        let mut inst = self.new_instruction(Box::new(Call {
            func,
            manager: InstManager::new(func.return_type.clone()),
        }));
        for arg in args {
            unsafe {
                inst.get_manager_mut().add_operand(arg);
            }
        }
        inst
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum ICmpOp {
    Eq,
    Ne,
    Slt,
    Sle,
    Sgt,
    Sge,
    Ult,
    Ule,
    Ugt,
    Uge,
}

impl Display for ICmpOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eq => write!(f, "eq"),
            Self::Ne => write!(f, "ne"),
            Self::Slt => write!(f, "slt"),
            Self::Sle => write!(f, "sle"),
            Self::Sgt => write!(f, "sgt"),
            Self::Sge => write!(f, "sge"),
            Self::Ult => write!(f, "ult"),
            Self::Ule => write!(f, "ule"),
            Self::Ugt => write!(f, "ugt"),
            Self::Uge => write!(f, "uge"),
        }
    }
}

pub struct ICmp {
    pub op: ICmpOp,
    /// The type of the compared value.
    pub comp_type: ValueType,
    manager: InstManager,
}

impl ICmp {
    pub fn get_lhs(&self) -> &Operand {
        &self.get_operand()[0]
    }
    pub fn get_rhs(&self) -> &Operand {
        &self.get_operand()[1]
    }

    /// # Safety
    ///
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn set_lhs(&mut self, operand: Operand) {
        self.manager.set_operand(0, operand);
    }

    /// # Safety
    ///
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn set_rhs(&mut self, operand: Operand) {
        self.manager.set_operand(1, operand);
    }
}

impl Display for ICmp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%icmp_{}", self.get_id())
    }
}

impl Instruction for ICmp {
    gen_common_code!(ICmp, ICmp);
    fn gen_llvm_ir(&self) -> String {
        format!(
            "{} = icmp {} {} {}, {}",
            self,
            self.op,
            self.comp_type,
            &self.get_operand()[0],
            &self.get_operand()[1]
        )
    }

    fn copy_self(&self) -> Box<dyn Instruction> {
        Box::new(ICmp {
            op: self.op,
            comp_type: self.comp_type.clone(),
            manager: InstManager::new(ValueType::Bool),
        })
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum FCmpOp {
    Oeq,
    Ogt,
    Oge,
    Olt,
    Ole,
    One,
    Ord,
    Ueq,
    Ugt,
    Uge,
    Ult,
    Ule,
    Une,
    Uno,
    False,
    True,
}

impl Display for FCmpOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Oeq => write!(f, "oeq"),
            Self::Ogt => write!(f, "ogt"),
            Self::Oge => write!(f, "oge"),
            Self::Olt => write!(f, "olt"),
            Self::Ole => write!(f, "ole"),
            Self::One => write!(f, "one"),
            Self::Ord => write!(f, "ord"),
            Self::Ueq => write!(f, "ueq"),
            Self::Ugt => write!(f, "ugt"),
            Self::Uge => write!(f, "uge"),
            Self::Ult => write!(f, "ult"),
            Self::Ule => write!(f, "ule"),
            Self::Une => write!(f, "une"),
            Self::Uno => write!(f, "uno"),
            Self::False => write!(f, "false"),
            Self::True => write!(f, "true"),
        }
    }
}

pub struct FCmp {
    pub op: FCmpOp,
    /// The type of the compared value.
    pub comp_type: ValueType,
    manager: InstManager,
}

impl FCmp {
    pub fn get_lhs(&self) -> &Operand {
        &self.get_operand()[0]
    }
    pub fn get_rhs(&self) -> &Operand {
        &self.get_operand()[1]
    }

    /// # Safety
    ///
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn set_lhs(&mut self, operand: Operand) {
        self.manager.set_operand(0, operand);
    }

    /// # Safety
    ///
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn set_rhs(&mut self, operand: Operand) {
        self.manager.set_operand(1, operand);
    }
}

impl Display for FCmp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%fcmp_{}", self.get_id())
    }
}

impl Instruction for FCmp {
    gen_common_code!(FCmp, FCmp);
    fn gen_llvm_ir(&self) -> String {
        format!(
            "{} = fcmp {} {} {}, {}",
            self,
            self.op,
            self.comp_type,
            &self.get_operand()[0],
            &self.get_operand()[1]
        )
    }

    fn copy_self(&self) -> Box<dyn Instruction> {
        Box::new(FCmp {
            op: self.op,
            comp_type: self.comp_type.clone(),
            manager: InstManager::new(ValueType::Bool),
        })
    }
}

pub struct Phi {
    incoming_values: Vec<(Operand, BBPtr)>,
    manager: InstManager,
}

impl Phi {
    pub fn get_incoming_values(&self) -> &[(Operand, BBPtr)] {
        &self.incoming_values
    }
    pub fn get_incoming_values_mut(&mut self) -> &mut [(Operand, BBPtr)] {
        &mut self.incoming_values
    }
    pub fn add_incoming_value(&mut self, val: Operand, pred: BBPtr) {
        self.incoming_values.push((val.clone(), pred));
        unsafe {
            self.get_manager_mut().add_operand(val);
        }
    }
    pub fn remove_incoming_value(&mut self, pred_id: usize) {
        let index = self
            .incoming_values
            .iter()
            .position(|(_, p)| p.id == pred_id)
            .unwrap();
        self.incoming_values.remove(index);
        unsafe {
            self.get_manager_mut().remove_operand(index);
        }
    }
    pub fn get_incoming_value(&self, bb: BBPtr) -> Option<&Operand> {
        for (val, pred) in &self.incoming_values {
            if *pred == bb {
                return Some(val);
            }
        }
        None
    }
}

impl Display for Phi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%phi_{}", self.get_id())
    }
}

impl Instruction for Phi {
    gen_common_code!(Phi, Phi);

    fn set_operand(&mut self, index: usize, operand: Operand) {
        unsafe {
            self.get_manager_mut().set_operand(index, operand.clone());
        }
        self.incoming_values[index].0 = operand;
    }

    fn gen_llvm_ir(&self) -> String {
        let mut res = format!("{} = phi {} ", self, self.get_value_type());
        for (op, bb) in self.get_incoming_values() {
            res.push_str(&format!("[{}, {}], ", op, bb.as_ref()));
        }
        res.truncate(res.len() - 2);
        res
    }

    fn copy_self(&self) -> Box<dyn Instruction> {
        Box::new(Phi {
            incoming_values: vec![],
            manager: InstManager::new(self.get_value_type().clone()),
        })
    }
}

pub struct Call {
    pub func: FunPtr,
    manager: InstManager,
}

impl Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%call_{}", self.get_id())
    }
}

impl Instruction for Call {
    gen_common_code!(Call, Call);
    fn gen_llvm_ir(&self) -> String {
        let value_type = self.get_value_type();
        let prefix = if value_type == ValueType::Void {
            String::new()
        } else {
            format!("{} = ", self)
        };
        let mut res = format!("{}call {} @{}(", prefix, value_type, &self.func.name);
        let operands = self.get_operand();
        for op in operands {
            res.push_str(&format!("{} {}, ", op.get_type(), op));
        }
        if !operands.is_empty() {
            res.truncate(res.len() - 2);
        }
        res.push(')');
        res
    }

    fn copy_self(&self) -> Box<dyn Instruction> {
        Box::new(Call {
            func: self.func,
            manager: InstManager::new(self.get_value_type()),
        })
    }
}
