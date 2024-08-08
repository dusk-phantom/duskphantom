use std::collections::{HashMap, HashSet};

use crate::{
    middle::{
        analysis::dominator_tree::DominatorTree,
        ir::{BBPtr, FunPtr, InstPtr},
        Program,
    },
    utils::mem::{ObjPool, ObjPtr},
};

use super::{alias_analysis::EffectRange, effect_analysis::EffectAnalysis};

pub type NodePtr = ObjPtr<Node>;

pub struct MemorySSA<'a> {
    builder: MemorySSABuilder,
    inst_to_node: HashMap<InstPtr, NodePtr>,
    block_to_node: HashMap<BBPtr, NodePtr>,
    node_to_user: HashMap<NodePtr, HashSet<NodePtr>>,
    effect_analysis: &'a EffectAnalysis,
    program: &'a Program,
}

#[allow(unused)]
impl<'a> MemorySSA<'a> {
    pub fn new(program: &'a Program, effect_analysis: &'a EffectAnalysis) -> Self {
        let mut memory_ssa = Self {
            builder: MemorySSABuilder {
                node_pool: ObjPool::new(),
                counter: 0,
            },
            inst_to_node: HashMap::new(),
            block_to_node: HashMap::new(),
            node_to_user: HashMap::new(),
            effect_analysis,
            program,
        };
        for func in program.module.functions.iter() {
            memory_ssa.run(*func);
        }
        memory_ssa
    }

    pub fn get_user(&self, node: NodePtr) -> HashSet<NodePtr> {
        self.node_to_user.get(&node).cloned().unwrap_or_default()
    }

    pub fn gen_llvm_ir(&self) -> String {
        let mut result = String::new();
        for func in self.program.module.functions.iter() {
            if func.is_lib() {
                continue;
            }
            result += &format!("MemorySSA for function: {}\n", func.name);
            for bb in func.dfs_iter() {
                result += &format!("{}:\n", bb.name);
                if let Some(node) = self.block_to_node.get(&bb) {
                    result += &self.gen_llvm_ir_node(*node);
                    result += "\n";
                }
                for inst in bb.iter() {
                    if let Some(node) = self.inst_to_node.get(&inst) {
                        result += &self.gen_llvm_ir_node(*node);
                        result += "\n";
                    }
                    result += &inst.gen_llvm_ir();
                    result += "\n";
                }
                result += "\n";
            }
        }
        result
    }

    pub fn gen_llvm_ir_node(&self, node: NodePtr) -> String {
        match node.as_ref() {
            Node::Entry(_) => "; 0".to_string(),
            Node::Normal(id, use_node, def_node, _) => {
                let mut result: Vec<String> = Vec::new();
                if let Some(use_node) = use_node {
                    result.push(format!("; MemoryUse({})", use_node.get_id()));
                }
                if let Some(def_node) = def_node {
                    result.push(format!("; {} = MemoryDef({})", id, def_node.get_id()));
                }
                result.join("\n")
            }
            Node::Phi(id, arg, _) => {
                let mut result = format!("; {} = MemoryPhi(", id);
                for (bb, node) in arg {
                    result.push_str(&format!("[{}, {}]", node.get_id(), bb.name));
                }
                result.push(')');
                result
            }
        }
    }

    fn run(&mut self, func: FunPtr) {
        let Some(entry) = func.entry else {
            return;
        };

        // Insert empty phi nodes
        let phi_insertions = self.insert_empty_phi(func);

        // Add entry node
        let mut range_to_node = RangeToNode::new();
        let entry_node = self.builder.get_entry();
        self.block_to_node.insert(entry, entry_node);
        range_to_node.insert(EffectRange::All, entry_node);

        // Add other nodes
        self.add_node_start_from(
            None,
            entry,
            &mut HashSet::new(),
            &mut range_to_node,
            &phi_insertions,
        )
    }

    fn add_node_start_from(
        &mut self,
        parent_bb: Option<BBPtr>,
        current_bb: BBPtr,
        visited: &mut HashSet<BBPtr>,
        range_to_node: &mut RangeToNode,
        phi_insertions: &HashMap<BBPtr, PhiInsertion>,
    ) {
        // Add argument for "phi" instruction
        if let Some(mut phi) = phi_insertions.get(&current_bb).and_then(|p| p.get()) {
            let value = range_to_node.get(phi.get_effect_range()).unwrap();
            phi.add_phi_arg((parent_bb.unwrap(), value));
            range_to_node.insert(phi.get_effect_range().clone(), phi);
        }

        // Do not continue if visited
        // Argument of "phi" instruction need to be added multiple times,
        // so that part is before this check
        if visited.contains(&current_bb) {
            return;
        }
        visited.insert(current_bb);

        // Build MemorySSA for each node
        for inst in current_bb.iter() {
            if let Some(effect) = self.effect_analysis.inst_effect.get(&inst) {
                let def_range = effect.def_range.clone().into();
                let use_range = effect.use_range.clone().into();
                let def_node = range_to_node.get(&def_range);
                let use_node = range_to_node.get(&use_range);
                let new_node = self.create_normal_node(use_node, def_node, inst);
                range_to_node.insert(def_range, new_node);
            }
        }

        // Visit all successors
        let successors = current_bb.get_succ_bb();
        for succ in successors {
            self.add_node_start_from(
                Some(current_bb),
                *succ,
                visited,
                &mut range_to_node.branch(),
                phi_insertions,
            );
        }
    }

    fn create_normal_node(
        &mut self,
        use_node: Option<NodePtr>,
        def_node: Option<NodePtr>,
        inst: InstPtr,
    ) -> NodePtr {
        let node = self.builder.get_normal_node(use_node, def_node, inst);
        self.inst_to_node.insert(inst, node);
        if let Some(use_node) = use_node {
            self.node_to_user.entry(use_node).or_default().insert(node);
        }
        node
    }

    /// Insert empty "phi" for basic blocks starting from `entry`
    /// Returns a mapping from basic block to phi insertions
    #[allow(unused)]
    fn insert_empty_phi(&mut self, func: FunPtr) -> HashMap<BBPtr, PhiInsertion> {
        let entry = func.entry.unwrap();
        let mut phi_insertions: HashMap<BBPtr, PhiInsertion> = HashMap::new();
        let mut dom_tree = DominatorTree::new(func);

        for bb in func.dfs_iter() {
            for inst in bb.iter() {
                if let Some(effect) = self.effect_analysis.inst_effect.get(&inst) {
                    // Only insert phi for stores
                    if effect.def_range.is_empty() {
                        continue;
                    }

                    // Insert phi with DFS on dominance frontier tree
                    let mut visited = HashSet::new();
                    let mut positions: Vec<(BBPtr, EffectRange)> = Vec::new();
                    positions.push((bb, effect.def_range.clone().into()));
                    while let Some((position, range)) = positions.pop() {
                        if visited.contains(&position) {
                            continue;
                        }
                        visited.insert(position);
                        let df = dom_tree.get_df(position);

                        // Insert phi for each dominance frontier, update effect range
                        for bb in df {
                            let phi = self.builder.get_phi(range.clone());
                            let phi = phi_insertions.entry(bb).or_default().insert(phi);
                            self.block_to_node.insert(bb, phi);
                            positions.push((bb, phi.get_effect_range().clone()));
                        }
                    }
                }
            }
        }

        // Return result
        phi_insertions
    }
}

struct MemorySSABuilder {
    node_pool: ObjPool<Node>,
    counter: usize,
}

impl MemorySSABuilder {
    fn new_node(&mut self, node: Node) -> NodePtr {
        self.node_pool.alloc(node)
    }

    fn next_counter(&mut self) -> usize {
        let counter = self.counter;
        self.counter += 1;
        counter
    }

    fn get_entry(&mut self) -> NodePtr {
        let next_counter = self.next_counter();
        self.new_node(Node::Entry(next_counter))
    }

    fn get_normal_node(
        &mut self,
        use_node: Option<NodePtr>,
        def_node: Option<NodePtr>,
        inst: InstPtr,
    ) -> NodePtr {
        let next_counter = self.next_counter();
        self.new_node(Node::Normal(next_counter, use_node, def_node, inst))
    }

    fn get_phi(&mut self, range: EffectRange) -> NodePtr {
        let next_counter = self.next_counter();
        self.new_node(Node::Phi(next_counter, Vec::new(), range))
    }
}

type PhiArg = (BBPtr, NodePtr);

#[allow(unused)]
pub enum Node {
    Entry(usize),
    Normal(usize, Option<NodePtr>, Option<NodePtr>, InstPtr),
    Phi(usize, Vec<PhiArg>, EffectRange),
}

impl Node {
    fn add_phi_arg(&mut self, arg: PhiArg) {
        match self {
            Node::Phi(_, args, _) => args.push(arg),
            _ => panic!("not a phi node"),
        }
    }

    fn get_effect_range(&self) -> &EffectRange {
        match self {
            Node::Phi(_, _, range) => range,
            _ => panic!("not a phi node"),
        }
    }

    fn merge_effect_range(&mut self, another: &EffectRange) {
        match self {
            Node::Phi(_, _, range) => range.merge(another),
            _ => panic!("not a phi node"),
        }
    }

    fn get_id(&self) -> usize {
        match self {
            Node::Entry(id) => *id,
            Node::Normal(id, _, _, _) => *id,
            Node::Phi(id, _, _) => *id,
        }
    }
}

pub struct PhiInsertion(Option<NodePtr>);

impl PhiInsertion {
    pub fn new() -> Self {
        Self(None)
    }

    /// Insert an empty phi node.
    /// Returns the inserted or merged phi node.
    pub fn insert(&mut self, phi: NodePtr) -> NodePtr {
        if let Some(node) = self.0.as_mut() {
            node.merge_effect_range(phi.get_effect_range());
            return *node;
        }
        self.0 = Some(phi);
        phi
    }

    /// Get containing phi node.
    pub fn get(&self) -> Option<NodePtr> {
        self.0
    }
}

impl Default for PhiInsertion {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct RangeToNodeFrame(Vec<(EffectRange, NodePtr)>);

impl RangeToNodeFrame {
    pub fn insert(&mut self, k: EffectRange, v: NodePtr) {
        self.0.push((k, v));
    }

    pub fn get(&self, k: &EffectRange) -> Option<NodePtr> {
        self.0.iter().rev().find_map(
            |(key, value)| {
                if key.can_alias(k) {
                    Some(*value)
                } else {
                    None
                }
            },
        )
    }
}

pub enum RangeToNode<'a> {
    Root(RangeToNodeFrame),
    Leaf(RangeToNodeFrame, &'a RangeToNode<'a>),
}

impl Default for RangeToNode<'_> {
    fn default() -> Self {
        Self::Root(RangeToNodeFrame::default())
    }
}

impl<'a> RangeToNode<'a> {
    /// Create a new FrameMap.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the last frame.
    pub fn last_frame(&mut self) -> &mut RangeToNodeFrame {
        match self {
            Self::Root(map) => map,
            Self::Leaf(map, _) => map,
        }
    }

    /// Insert a new element into the last frame.
    pub fn insert(&mut self, k: EffectRange, v: NodePtr) {
        if k.is_empty() {
            return;
        }
        self.last_frame().insert(k, v);
    }

    /// Get an element from all frames.
    pub fn get(&self, k: &EffectRange) -> Option<NodePtr> {
        if k.is_empty() {
            return None;
        }
        let mut map = self;
        loop {
            match map {
                Self::Root(m) => return m.get(k),
                Self::Leaf(m, parent) => {
                    if let Some(v) = m.get(k) {
                        return Some(v);
                    }
                    map = parent;
                }
            }
        }
    }

    /// Make a branch on the frame map.
    /// Modifications on the new branch will not affect the original one.
    /// This is useful when implementing scopes.
    pub fn branch(&'a self) -> Self {
        Self::Leaf(RangeToNodeFrame::default(), self)
    }
}
