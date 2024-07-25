use crate::frontend::Decl;
use crate::middle::ir::ValueType;
use crate::middle::irgen::function_kit::FunctionKit;
use crate::middle::irgen::program_kit::ProgramKit;
use crate::middle::irgen::value::Value;
use crate::{errors::MiddleError, middle::ir::InstPtr};

use super::function_kit::{FunctionContext, FunctionRouting};

impl<'a> ProgramKit<'a> {
    /// Generate an implementation into the program
    pub fn gen_impl(&mut self, decl: &Decl) -> anyhow::Result<()> {
        match decl {
            Decl::Func(_, id, Some(stmt)) => {
                // Get function and its type
                let mut fun_ptr = self.fun_env.get(id).copied().ok_or(MiddleError::GenError)?;
                let fty = fun_ptr.return_type.clone();

                // Create basic block
                let entry_name = "entry".to_string();
                let mut entry = self.program.mem_pool.new_basicblock(entry_name);
                let exit_name = "exit".to_string();
                let mut exit = self.program.mem_pool.new_basicblock(exit_name);

                // Handle value to return
                let mut ret_value = None;
                if let ValueType::Void = fty {
                    let ret_inst = self.program.mem_pool.get_ret(None);
                    exit.push_back(ret_inst);
                } else {
                    let ret_alloc = self.program.mem_pool.get_alloca(fty.clone(), 1);
                    entry.push_back(ret_alloc);
                    let ret_load = self
                        .program
                        .mem_pool
                        .get_load(fty.clone(), ret_alloc.into());
                    exit.push_back(ret_load);
                    let ret_inst = self.program.mem_pool.get_ret(Some(ret_load.into()));
                    exit.push_back(ret_inst);
                    ret_value = Some(Value::ReadWrite(ret_alloc.into()));
                }

                // Build function
                let mut counter: usize = 0;
                let mut kit = FunctionKit::new(
                    FunctionContext {
                        env: self.env.branch(),
                        fun_env: self.fun_env.branch(),
                        program: self.program,
                        counter: &mut counter,
                    },
                    FunctionRouting {
                        exit: Some(entry),
                        break_to: None,
                        continue_to: None,
                        return_to: exit,
                        return_value: ret_value,
                        return_type: fty,
                    },
                );

                // Fill parameters
                for param in fun_ptr.params.iter() {
                    // Add parameter to entry
                    let value_type = param.value_type.clone();
                    let value = (*param).into();
                    let alloc: InstPtr = kit.program.mem_pool.get_alloca(value_type, 1);
                    let store: InstPtr = kit.program.mem_pool.get_store(value, alloc.into());
                    entry.push_back(alloc);
                    entry.push_back(store);

                    // Add parameter to function env
                    kit.env
                        .insert(param.name.clone(), Value::ReadWrite(alloc.into()));
                }

                // Generate statements
                kit.gen_stmt(stmt)?;

                // If kit still has exit, it did not return, so redirect it to return
                if let Some(mut kit_exit) = kit.exit {
                    kit_exit.push_back(kit.program.mem_pool.get_br(None));
                    kit_exit.set_true_bb(exit);
                }

                // Set function entry and exit
                fun_ptr.entry = Some(entry);
                fun_ptr.exit = Some(exit);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
