// use std::collections::{HashMap, HashSet};

// use crate::{
//     backend::from_self::downcast_ref,
//     middle::{
//         ir::{
//             instruction::{misc_inst::Call, InstType},
//             FunPtr, InstPtr, Operand,
//         },
//         Program,
//     },
//     utils::traverse::{Node, POIterator},
// };

// pub struct Effect {
//     mem_def: HashMap<FunPtr, HashSet<Operand>>,
//     mem_use: HashMap<FunPtr, HashSet<Operand>>,
// }

// impl Effect {
//     pub fn new(program: &Program) -> Self {
//         let main_fun = program
//             .module
//             .functions
//             .iter()
//             .find(|f| f.name == "main")
//             .cloned()
//             .unwrap();
//         Self { main_fun }
//     }
// }
