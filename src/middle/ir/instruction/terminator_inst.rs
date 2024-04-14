use super::*;

pub struct Ret {
    manager: InstManager,
}

pub struct Br {
    manager: InstManager,
}

impl IRBuilder {
    pub fn get_ret(&mut self, return_value: Option<Operand>) -> InstPtr {
        let mut ret = self.new_instruction(Box::new(Ret {
            manager: InstManager::new(return_value.as_ref().map_or(ValueType::Void, |x| x.get_type())),
        }));
        return_value.map(|x| unsafe {
            ret.get_manager_mut().add_operand(x);
        });
        ret
    }

    pub fn get_br(&mut self, cond: Option<Operand>) -> InstPtr {
        let mut br = self.new_instruction(Box::new(Br {
            manager: InstManager::new(ValueType::Void),
        }));
        cond.map(|x| unsafe {
            br.get_manager_mut().add_operand(x);
        });
        br
    }
}

impl Ret {
    pub fn is_void(&self) -> bool {
        self.manager.operand.len() == 0
    }

    pub fn get_return_value(&self) -> &Operand {
        &self.manager.operand[0]
    }
}

impl Br {
    pub fn is_cond_br(&self) -> bool {
        self.manager.operand.len() == 1
    }
    pub fn get_cond(&self) -> &Operand {
        &self.manager.operand[0]
    }
}

impl Display for Ret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%ret_{}", self.get_id())
    }
}

impl Display for Br {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%br_{}", self.get_id())
    }
}

impl Instruction for Ret {
    gen_common_code!(Ret, Ret);
    #[inline]
    fn gen_llvm_ir(&self) -> String {
        if self.is_void() {
            format!("ret void")
        } else {
            format!("ret {}", self.get_return_value())
        }
    }
}

impl Instruction for Br {
    gen_common_code!(Br, Br);
    #[inline]
    fn gen_llvm_ir(&self) -> String {
        let parent_bb = self.get_parent_bb().unwrap();
        let next_bb = parent_bb.get_succ_bb();
        if self.is_cond_br() {
            format!(
                "br i1 {}, label %{}, label %{}",
                self.get_cond(),
                next_bb[0].name,
                next_bb[1].name
            )
        } else {
            format!("br label %{}", next_bb[0].name)
        }
    }
}
