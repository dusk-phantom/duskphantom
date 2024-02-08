#[macro_export]
macro_rules! define_graph_iterator {
    ($name:ident, $collection:ty, $pop_method:ident, $bb_update_method:ident) => {
        pub struct $name {
            container: $collection,
            visited: HashSet<BBPtr>,
        }

        impl Iterator for $name {
            type Item = BBPtr;
            fn next(&mut self) -> Option<Self::Item> {
                while let Some(bb) = self.container.$pop_method() {
                    if !self.visited.contains(&bb) {
                        self.visited.insert(bb);
                        self.container.extend(bb.$bb_update_method());
                        return Some(bb);
                    }
                }
                None
            }
        }

        impl From<BBPtr> for $name {
            fn from(bb: BBPtr) -> Self {
                Self {
                    container: vec![bb].into(),
                    visited: HashSet::new(),
                }
            }
        }
    };
}

/// Use this macro to generate some code for instruction to simplify the codesize.
/// Make sure to use this macro in the impl Instruction block
/// and the Instruction struct must have a field named manager
/// which is an instance of InstManager.
#[macro_export]
macro_rules! gen_common_code {
    ($type:ty,$id:ident) => {
        #[inline]
        unsafe fn as_any(&self) -> &dyn Any {
            self
        }
        #[inline]
        unsafe fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        #[inline]
        fn get_type(&self) -> InstType {
            InstType::$id
        }
        #[inline]
        fn get_manager(&self) -> &InstManager {
            &self.manager
        }
        #[inline]
        unsafe fn get_manager_mut(&mut self) -> &mut InstManager {
            &mut self.manager
        }
    };
}

/// impl InstType enum automatically.
#[macro_export]
macro_rules! define_inst_type_enum {
    ($( $variant:ident ),*) => {
        #[derive(Clone, Copy, Eq, PartialEq)]
        pub enum InstType {
            $( $variant ),*
        }

        impl InstType {
            #[inline]
            fn get_name(&self) -> String {
                match self {
                    $( InstType::$variant => stringify!($variant).to_lowercase(), )*
                }
            }
        }

        impl std::fmt::Display for InstType {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.get_name())
            }
        }

        impl std::fmt::Debug for InstType {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.get_name())
            }
        }
    };
}

/// impl BinaryInst trait automatically.
#[macro_export]
macro_rules! impl_binary_inst {
    ($type:ident, $operand_type:expr,$func: ident, $lhs:ident, $rhs: ident) => {
        /// If you want to make a new binary inst,
        /// please use the IRBuilder to create it.
        pub struct $type {
            manager: InstManager,
        }
        impl BinaryInst for $type {
            #[inline]
            fn get_lhs(&self) -> InstPtr {
                self.manager.operand[0]
            }

            #[inline]
            fn set_lhs(&mut self, lhs: InstPtr) {
                self.manager.operand[0] = lhs;
            }

            #[inline]
            fn get_rhs(&self) -> InstPtr {
                self.manager.operand[1]
            }

            #[inline]
            fn set_rhs(&mut self, rhs: InstPtr) {
                self.manager.operand[1] = rhs;
            }
        }

        impl Instruction for $type {
            gen_common_code!($type, $type);
            #[inline]
            fn gen_llvm_ir(&self) -> String {
                format!(
                    "%{} = {} {}, %{}, %{}",
                    self.get_id(),
                    self.get_type(),
                    $operand_type,
                    self.get_lhs().get_id(),
                    self.get_rhs().get_id()
                )
            }
        }

        impl IRBuilder {
            /// Get a new inst instruction with operands.
            pub fn $func(&mut self, $lhs: InstPtr, $rhs: InstPtr) -> InstPtr {
                let mut inst = $type {
                    manager: InstManager::new(self.new_inst_id()),
                };
                inst.set_lhs($lhs);
                inst.set_rhs($rhs);
                self.new_instruction(Box::new(inst))
            }
        }
    };
}
