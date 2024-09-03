// Copyright 2024 Duskphantom Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

pub struct IRBuilder {
    fun_pool: ObjPool<Function>,
    bb_pool: ObjPool<BasicBlock>,
    inst_pool: ObjPool<Box<dyn Instruction>>,
    gvar_pool: ObjPool<GlobalVariable>,
    param_pool: ObjPool<Parameter>,
    inst_id: usize,
    block_id: usize,
}

impl Default for IRBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl IRBuilder {
    pub fn new() -> Self {
        Self {
            fun_pool: ObjPool::new(),
            bb_pool: ObjPool::new(),
            inst_pool: ObjPool::new(),
            gvar_pool: ObjPool::new(),
            param_pool: ObjPool::new(),
            inst_id: 0,
            block_id: 1,
        }
    }

    /// Allocate a space for global variable, return a pointer to this space.
    pub fn new_global_variable(
        &mut self,
        name: String,
        value_type: ValueType,
        variable_or_constant: bool,
        initializer: Constant,
    ) -> GlobalPtr {
        self.gvar_pool
            .alloc(GlobalVariable::new(
                name,
                value_type,
                variable_or_constant,
                initializer,
            ))
            .into()
    }

    /// Copy a global variable
    pub fn copy_global_variable(&mut self, new_name: String, global: GlobalPtr) -> GlobalPtr {
        self.gvar_pool
            .alloc(GlobalVariable::new(
                new_name,
                global.as_ref().value_type.clone(),
                global.as_ref().variable_or_constant,
                global.as_ref().initializer.clone(),
            ))
            .into()
    }

    /// Allocate a space for func, return a pointer to this space.
    pub fn new_function(&mut self, name: String, return_type: ValueType) -> FunPtr {
        let func = Function {
            mem_pool: self.into(),
            name: name.clone(),
            entry: None,
            exit: None,
            return_type,
            params: Vec::new(),
        };
        self.fun_pool.alloc(func)
    }

    /// Allocate a space for basicblock, return a pointer to this space.
    pub fn new_basicblock(&mut self, name: String) -> BBPtr {
        let ptr = self.into();
        let bb = self.bb_pool.alloc(BasicBlock::new(name, ptr));
        unsafe {
            BasicBlock::init_bb(bb, self.new_block_id());
        }
        bb
    }

    /// Gets a new id for instruction.
    #[inline]
    fn new_inst_id(&mut self) -> usize {
        let old = self.inst_id;
        self.inst_id += 1;
        old
    }

    #[inline]
    fn new_block_id(&mut self) -> usize {
        let old = self.block_id;
        self.block_id += 1;
        old
    }

    /// Allocate a space for instruction, return a pointer to this space.
    pub fn new_instruction(&mut self, inst: Box<dyn Instruction>) -> InstPtr {
        let mut inst = self.inst_pool.alloc(inst);
        let id = self.new_inst_id();
        unsafe {
            inst.get_manager_mut().set_id(id);
            let ic = inst;
            inst.get_manager_mut().set_self_ptr(ic.into());
        }
        inst.into()
    }

    /// Copy a instruction.
    ///
    /// As `Instruction::copy_self` can't access self_ptr,
    /// operand can't be copied in copy_self with use-def chain maintained.
    ///
    /// `copy_instruction` does not copy operand, because
    /// copied instruction's operand is usually reset anyway.
    pub fn copy_instruction(&mut self, inst: &dyn Instruction) -> InstPtr {
        unsafe {
            let mut inst = self.inst_pool.alloc(inst.copy_self());
            let id = self.new_inst_id();
            inst.get_manager_mut().set_id(id);
            let ic = inst;
            inst.get_manager_mut().set_self_ptr(ic.into());
            inst.into()
        }
    }

    /// Allocate a space for parameter, return a pointer to this space.
    pub fn new_parameter(&mut self, name: String, value_type: ValueType) -> ParaPtr {
        self.param_pool
            .alloc(Parameter::new(name, value_type))
            .into()
    }
}

impl IRBuilder {
    pub fn clear(&mut self) {
        self.fun_pool.clear();
        self.bb_pool.clear();
        self.inst_pool.clear();
    }
}
