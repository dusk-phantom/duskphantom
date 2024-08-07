use crate::middle::ir::BBPtr;
use crate::middle::ir::FunPtr;
use std::collections::{HashMap, HashSet};

pub mod call_graph;
pub mod dominator_tree;
pub mod loop_tools;
pub mod reachability;
