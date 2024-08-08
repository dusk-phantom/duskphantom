use crate::middle::ir::BBPtr;
use crate::middle::ir::FunPtr;
use std::collections::{HashMap, HashSet};

pub mod alias_analysis;
pub mod call_graph;
pub mod dominator_tree;
pub mod effect_analysis;
pub mod loop_tools;
pub mod reachability;
