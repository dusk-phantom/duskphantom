use crate::{
    backend::from_self::downcast_ref,
    middle::{
        ir::{
            instruction::{misc_inst::Call, InstType},
            FunPtr,
        },
        Program,
    },
    utils::traverse::{Node, POIterator},
};

pub fn build_call_graph(program: &Program) -> POIterator<CallGraphNode> {
    let main_fun = program
        .module
        .functions
        .iter()
        .find(|f| f.name == "main")
        .cloned()
        .unwrap();
    let node = CallGraphNode { fun: main_fun };
    node.into()
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CallGraphNode {
    fun: FunPtr,
}

impl Node for CallGraphNode {
    fn get_succ(&self) -> Vec<Self> {
        let mut result = Vec::new();
        for bb in self.fun.dfs_iter() {
            for inst in bb.iter() {
                if inst.get_type() == InstType::Call {
                    let call = downcast_ref::<Call>(inst.as_ref().as_ref());
                    result.push(CallGraphNode { fun: call.func });
                }
            }
        }
        result
    }
}
