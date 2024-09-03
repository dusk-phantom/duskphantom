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

impl IRBuilder {
    /// Create a new `Alloca` instruction.
    /// The `Alloca` instruction is used to allocate memory on the stack.
    ///
    /// # Arguments
    /// * `value_type` - The type of the value to be allocated
    /// * `num_elements` - The number of elements to be allocated
    ///
    /// # Return
    /// The pointer to the `Alloca` instruction
    ///
    /// # Example
    /// ```rust
    /// # use duskphantom_middle::ir::*;
    /// let mut ir_builder = IRBuilder::new();
    /// let alloca_0 = ir_builder.get_alloca(ValueType::Int, 1);// %alloca_0 = alloca i32
    /// let alloca_1 = ir_builder.get_alloca(ValueType::Int, 2);// %alloca_1 = alloca i32, i32 2
    /// ```
    pub fn get_alloca(&mut self, value_type: ValueType, num_elements: usize) -> InstPtr {
        self.new_instruction(Box::new(Alloca {
            value_type: value_type.clone(),
            num_elements,
            manager: InstManager::new(ValueType::Pointer(Box::new(value_type))),
        }))
    }

    /// Create a new `Load` instruction.
    /// The `Load` instruction is used to load a value from memory.
    ///
    /// # Arguments
    /// * `value_type` - The type of the value to be loaded
    /// * `ptr` - The pointer to the value to be loaded
    ///
    /// # Return
    /// The pointer to the `Load` instruction
    ///
    /// # Example
    /// ```rust
    /// # use duskphantom_middle::ir::*;
    /// let mut ir_builder = IRBuilder::new();
    /// let ptr = ir_builder.get_alloca(ValueType::Int, 1);// %ptr = alloca i32
    /// let load_0 = ir_builder.get_load(ValueType::Int, Operand::Instruction(ptr));// %load_0 = load i32, ptr
    /// ```
    pub fn get_load(&mut self, value_type: ValueType, ptr: Operand) -> InstPtr {
        let mut inst = self.new_instruction(Box::new(Load {
            manager: InstManager::new(value_type),
        }));
        unsafe { inst.get_manager_mut().add_operand(ptr) };
        inst
    }

    /// Create a new `Store` instruction.
    /// The `Store` instruction is used to store a value to memory.
    ///
    /// # Arguments
    /// * `value` - The value to be stored
    /// * `ptr` - The pointer to the value to be stored
    ///
    /// # Return
    /// The pointer to the `Store` instruction
    ///
    /// # Example
    /// ```rust
    /// # use duskphantom_middle::ir::*;
    /// let mut ir_builder = IRBuilder::new();
    /// let ptr = ir_builder.get_alloca(ValueType::Int, 1);// %ptr = alloca i32
    /// let value = Operand::Constant(1.into());// %value = 1
    /// let store_0 = ir_builder.get_store(value, Operand::Instruction(ptr));// store i32 %value, ptr %ptr
    /// ```
    pub fn get_store(&mut self, value: Operand, ptr: Operand) -> InstPtr {
        let mut inst = self.new_instruction(Box::new(Store {
            manager: InstManager::new(ValueType::Void),
        }));
        unsafe {
            inst.get_manager_mut().add_operand(value);
            inst.get_manager_mut().add_operand(ptr);
        }
        inst
    }

    /// Create a new `GetElementPtr` instruction.
    /// The `GetElementPtr` instruction is used to get the pointer to an element of an array or a struct.
    ///
    /// # Arguments
    /// * `element_type` - The type of the element to be accessed
    /// * `ptr` - The pointer to the array or the struct
    /// * `index` - The index of the element to be accessed
    ///
    /// # Return
    /// The pointer to the `GetElementPtr` instruction
    ///
    /// # Panics
    /// Attempt to access subtype of primitive pointers will panic.
    ///
    /// # Example
    /// ```rust
    /// # use duskphantom_middle::ir::*;
    /// let mut ir_builder = IRBuilder::new();
    /// let ptr = ir_builder.get_alloca(ValueType::Int, 1);// %ptr = alloca i32
    /// let index = vec![Operand::Constant(1.into())];// %index = 1
    /// let getelementptr_0 = ir_builder.get_getelementptr(ValueType::Int, Operand::Instruction(ptr), index);// %getelementptr_0 = getelementptr i32, ptr %ptr, i32 %index
    /// ```
    pub fn get_getelementptr(
        &mut self,
        element_type: ValueType,
        ptr: Operand,
        index: Vec<Operand>,
    ) -> InstPtr {
        // Calculate return type by getting subtype for each non-initial index.
        // For example, in `getelementptr [2 x i32], ptr %alloca_0, i32 1, i32 1`
        // the second index shrinks the pointer type to `i32*`.
        let mut return_type = element_type.clone();
        for _ in &index[1..] {
            if let Some(sub_type) = return_type.get_sub_type() {
                return_type = sub_type.clone();
            } else {
                panic!("Invalid index for getelementptr instruction");
            }
        }

        // Generate the instruction from calculated type
        let mut inst = self.new_instruction(Box::new(GetElementPtr {
            element_type,
            manager: InstManager::new(ValueType::Pointer(Box::new(return_type))),
        }));
        unsafe {
            inst.get_manager_mut().add_operand(ptr);
            for i in index {
                inst.get_manager_mut().add_operand(i);
            }
        }
        inst
    }
}

pub struct Alloca {
    /// The type of the value to be allocated
    pub value_type: ValueType,

    /// The number of elements to be allocated
    pub num_elements: usize,

    manager: InstManager,
}

impl Display for Alloca {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%alloca_{}", self.get_id())
    }
}

impl Instruction for Alloca {
    gen_common_code!(Alloca, Alloca);
    fn gen_llvm_ir(&self) -> String {
        format!(
            "{} = alloca {}{}",
            self,
            self.value_type,
            if self.num_elements == 1 {
                "".to_string()
            } else {
                format!(" , i32 {}", self.num_elements)
            }
        )
    }

    fn copy_self(&self) -> Box<dyn Instruction> {
        Box::new(Alloca {
            value_type: self.value_type.clone(),
            num_elements: self.num_elements,
            manager: InstManager::new(self.get_value_type()),
        })
    }
}

pub struct Load {
    manager: InstManager,
}

impl Load {
    /// Get the pointer to the value to be loaded
    /// # Return
    /// The pointer to the value to be loaded
    pub fn get_ptr(&self) -> &Operand {
        &self.get_operand()[0]
    }

    /// # Safety
    ///
    /// Set the pointer to the value to be loaded
    ///
    /// # Arguments
    ///
    /// * `ptr` - The pointer to the value to be loaded
    pub unsafe fn set_ptr(&mut self, ptr: Operand) {
        self.get_manager_mut().set_operand(0, ptr);
    }
}

impl Display for Load {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%load_{}", self.get_id())
    }
}

impl Instruction for Load {
    gen_common_code!(Load, Load);
    fn gen_llvm_ir(&self) -> String {
        format!(
            "{} = load {}, ptr {}",
            self,
            self.get_value_type(),
            self.get_operand()[0]
        )
    }

    fn copy_self(&self) -> Box<dyn Instruction> {
        Box::new(Load {
            manager: InstManager::new(self.get_value_type()),
        })
    }
}

pub struct Store {
    manager: InstManager,
}

impl Store {
    /// Get the value to be stored
    /// # Return
    /// The value to be stored
    pub fn get_value(&self) -> &Operand {
        &self.get_operand()[0]
    }

    /// Get the pointer to the value to be stored
    /// # Return
    /// The pointer to the value to be stored
    pub fn get_ptr(&self) -> &Operand {
        &self.get_operand()[1]
    }

    /// # Safety
    ///
    /// Set the value to be stored
    /// # Arguments
    /// * `value` - The value to be stored
    pub unsafe fn set_value(&mut self, value: Operand) {
        self.get_manager_mut().set_operand(0, value);
    }

    /// # Safety
    ///
    /// Set the pointer to the value to be stored
    /// # Arguments
    /// * `ptr` - The pointer to the value to be stored
    pub unsafe fn set_ptr(&mut self, ptr: Operand) {
        self.get_manager_mut().set_operand(1, ptr);
    }
}

impl Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%store_{}", self.get_id())
    }
}

impl Instruction for Store {
    gen_common_code!(Store, Store);
    fn gen_llvm_ir(&self) -> String {
        format!(
            "store {} {}, ptr {}",
            match &self.get_operand()[0] {
                Operand::Instruction(inst) => format!("{}", inst.get_value_type()),
                Operand::Constant(c) => format!("{}", c.get_type()),
                Operand::Parameter(param) => format!("{}", param.value_type),
                _ => panic!("Invalid operand type"),
            },
            self.get_operand()[0],
            self.get_operand()[1]
        )
    }

    fn copy_self(&self) -> Box<dyn Instruction> {
        Box::new(Store {
            manager: InstManager::new(ValueType::Void),
        })
    }
}

/// The `GetElementPtr` instruction is used to get the pointer to an element of an array or a struct.
/// See also: [LLVM GetElementPtr](https://llvm.org/docs/GetElementPtr.html)
pub struct GetElementPtr {
    /// The type of the element to be accessed
    pub element_type: ValueType,
    manager: InstManager,
}

impl GetElementPtr {
    /// Get the pointer to the array or the struct
    /// # Return
    /// The pointer to the array or the struct
    pub fn get_ptr(&self) -> &Operand {
        &self.get_operand()[0]
    }

    /// # Safety
    ///
    /// Set the pointer to the array or the struct
    /// # Arguments
    /// * `ptr` - The pointer to the array or the struct
    pub unsafe fn set_ptr(&mut self, ptr: Operand) {
        self.get_manager_mut().set_operand(0, ptr);
    }

    /// Get the index of the element to be accessed
    /// # Return
    /// The index of the element to be accessed
    pub fn get_index(&self) -> &[Operand] {
        &self.get_operand()[1..]
    }

    /// # Safety
    ///
    /// Set the index of the element to be accessed
    /// # Arguments
    /// * `index` - The index of the element to be accessed
    pub unsafe fn set_index(&mut self, index: Vec<Operand>) {
        for (i, idx) in index.into_iter().enumerate() {
            self.get_manager_mut().set_operand(i + 1, idx);
        }
    }
}

impl Display for GetElementPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%getelementptr_{}", self.get_id())
    }
}

impl Instruction for GetElementPtr {
    gen_common_code!(GetElementPtr, GetElementPtr);
    fn gen_llvm_ir(&self) -> String {
        let mut s = format!(
            "{} = getelementptr {}, ptr {}",
            self,
            self.element_type,
            self.get_operand()[0]
        );
        if self.get_operand().len() > 1 {
            for index in &self.get_operand()[1..] {
                s.push_str(&format!(", i32 {}", index));
            }
        }
        s
    }

    fn copy_self(&self) -> Box<dyn Instruction> {
        Box::new(GetElementPtr {
            element_type: self.element_type.clone(),
            manager: InstManager::new(self.get_value_type()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloca() {
        let mut ir_builder = IRBuilder::new();
        let alloca = ir_builder.get_alloca(ValueType::Int, 1);
        assert_eq!(alloca.to_string(), "%alloca_0");
        assert_eq!(
            alloca.get_value_type(),
            ValueType::Pointer(Box::new(ValueType::Int))
        );
        assert_eq!(alloca.gen_llvm_ir(), "%alloca_0 = alloca i32");
    }

    #[test]
    fn test_load() {
        let mut ir_builder = IRBuilder::new();
        let ptr = ir_builder.get_alloca(ValueType::Int, 1);
        let load = ir_builder.get_load(ValueType::Int, Operand::Instruction(ptr));
        assert_eq!(load.to_string(), "%load_1");
        assert_eq!(load.get_value_type(), ValueType::Int);
        assert_eq!(load.gen_llvm_ir(), "%load_1 = load i32, ptr %alloca_0");
    }

    #[test]
    fn test_store() {
        let mut ir_builder = IRBuilder::new();
        let ptr = ir_builder.get_alloca(ValueType::Int, 1);
        let value = Operand::Constant(1.into());
        let store = ir_builder.get_store(value, Operand::Instruction(ptr));
        assert_eq!(store.to_string(), "%store_1");
        assert_eq!(store.get_value_type(), ValueType::Void);
        assert_eq!(store.gen_llvm_ir(), "store i32 1, ptr %alloca_0");
    }

    #[test]
    fn test_getelementptr() {
        let mut ir_builder = IRBuilder::new();
        let ptr = ir_builder.get_alloca(ValueType::Int, 1);
        let index = vec![Operand::Constant(1.into())];
        let getelementptr =
            ir_builder.get_getelementptr(ValueType::Int, Operand::Instruction(ptr), index);
        assert_eq!(getelementptr.to_string(), "%getelementptr_1");
        assert_eq!(
            getelementptr.get_value_type(),
            ValueType::Pointer(Box::new(ValueType::Int))
        );
        assert_eq!(
            getelementptr.gen_llvm_ir(),
            "%getelementptr_1 = getelementptr i32, ptr %alloca_0, i32 1"
        );
    }

    #[test]
    fn test_getelementptr_subtype() {
        let mut ir_builder = IRBuilder::new();
        let array_value = Constant::Array(vec![Constant::Int(1), Constant::Int(2)]);
        let array_type = ValueType::Array(ValueType::Int.into(), 2);
        let ptr =
            ir_builder.new_global_variable("a".to_string(), array_type.clone(), true, array_value);
        let index = vec![Operand::Constant(1.into()), Operand::Constant(1.into())];
        let getelementptr = ir_builder.get_getelementptr(array_type, Operand::Global(ptr), index);
        assert_eq!(getelementptr.to_string(), "%getelementptr_0");
        assert_eq!(
            getelementptr.get_value_type(),
            ValueType::Pointer(Box::new(ValueType::Int))
        );
        assert_eq!(
            getelementptr.gen_llvm_ir(),
            "%getelementptr_0 = getelementptr [2 x i32], ptr @a, i32 1, i32 1"
        );
    }
}
