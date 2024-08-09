use std::collections::{HashMap, HashSet};

use crate::{
    backend::from_self::downcast_ref,
    middle::{
        analysis::dominator_tree::DominatorTree,
        ir::{
            instruction::{misc_inst::Call, InstType},
            BBPtr, FunPtr, InstPtr,
        },
        Program,
    },
    utils::mem::{ObjPool, ObjPtr},
};

use super::{alias_analysis::EffectRange, effect_analysis::EffectAnalysis};

pub type NodePtr = ObjPtr<Node>;

/// MemorySSA analyzer.
/// Reference: https://llvm.org/docs/MemorySSA.html
/// My version is different by analyzing the effect of function calls.
pub struct MemorySSA<'a> {
    builder: MemorySSABuilder,
    functions: Vec<FunPtr>,
    inst_to_node: HashMap<InstPtr, NodePtr>,
    block_to_node: HashMap<BBPtr, NodePtr>,
    node_to_user: HashMap<NodePtr, HashSet<NodePtr>>,
    effect_analysis: &'a EffectAnalysis,
}

impl<'a> MemorySSA<'a> {
    /// Build MemorySSA for program.
    pub fn new(program: &Program, effect_analysis: &'a EffectAnalysis) -> Self {
        let mut memory_ssa = Self {
            builder: MemorySSABuilder {
                node_pool: ObjPool::new(),
                counter: 0,
            },
            inst_to_node: HashMap::new(),
            block_to_node: HashMap::new(),
            node_to_user: HashMap::new(),
            effect_analysis,
            functions: program.module.functions.clone(),
        };
        for func in program.module.functions.iter() {
            memory_ssa.run(*func);
        }
        memory_ssa
    }

    /// Get node from instruction.
    pub fn get_inst_node(&self, inst: InstPtr) -> Option<NodePtr> {
        self.inst_to_node.get(&inst).cloned()
    }

    /// Get node from block.
    pub fn get_block_node(&self, bb: BBPtr) -> Option<NodePtr> {
        self.block_to_node.get(&bb).cloned()
    }

    /// Get all users of a node.
    pub fn get_user(&self, node: NodePtr) -> HashSet<NodePtr> {
        self.node_to_user.get(&node).cloned().unwrap_or_default()
    }

    /// Dump MemorySSA result to string.
    pub fn dump(&self) -> String {
        let mut result = String::new();
        for func in self.functions.iter() {
            if func.is_lib() {
                continue;
            }
            result += &format!("MemorySSA for function: {}\n", func.name);
            for bb in func.dfs_iter() {
                result += &format!("{}:\n", bb.name);
                if let Some(node) = self.block_to_node.get(&bb) {
                    result += &self.dump_node(*node);
                    result += "\n";
                }
                for inst in bb.iter() {
                    if let Some(node) = self.inst_to_node.get(&inst) {
                        result += &self.dump_node(*node);
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

    /// Dump a node to string.
    pub fn dump_node(&self, node: NodePtr) -> String {
        match node.as_ref() {
            Node::Entry(id) => format!("; {} (liveOnEntry)", id),
            Node::Normal(id, used_node, def_node, _, _) => {
                let mut result: Vec<String> = Vec::new();
                if let Some(used_node) = used_node {
                    result.push(format!("; MemoryUse({})", used_node.get_id()));
                }
                if let Some(def_node) = def_node {
                    result.push(format!("; {} = MemoryDef({})", id, def_node.get_id()));
                }
                result.join("\n")
            }
            Node::Phi(id, arg, _) => {
                let mut args: Vec<String> = Vec::new();
                for (bb, node) in arg {
                    args.push(format!("[{}, {}]", node.get_id(), bb.name));
                }
                format!("; {} = MemoryPhi({})", id, args.join(", "))
            }
        }
    }

    /// Remove a node, update use-def chain.
    ///
    /// # Panics
    /// If the node is used, it will cause undefined behaviour.
    pub fn remove_node(&mut self, node: NodePtr) {
        let used_nodes = node.get_used_node();
        for used_node in &used_nodes {
            self.node_to_user.get_mut(used_node).unwrap().remove(&node);
        }
    }

    /// Build MemorySSA for function.
    fn run(&mut self, func: FunPtr) {
        let Some(entry) = func.entry else {
            return;
        };

        // Add entry node
        let mut range_to_node = RangeToNode::new();
        let entry_node = self.builder.get_entry();
        self.block_to_node.insert(entry, entry_node);
        range_to_node.insert(EffectRange::All, entry_node);

        // Insert empty phi nodes
        let phi_insertions = self.insert_empty_phi(func);

        // Add other nodes
        self.add_node_start_from(
            None,
            entry,
            &mut HashSet::new(),
            &mut range_to_node,
            &phi_insertions,
        )
    }

    /// Add nodes starting from `current_bb`.
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
            let value = range_to_node.get_def(phi.get_effect_range()).unwrap();
            phi.add_phi_arg((parent_bb.unwrap(), value));
            self.node_to_user.entry(value).or_default().insert(phi);
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
                let def_range = effect.def_range.clone();
                let use_range = effect.use_range.clone();
                let def_node = range_to_node.get_def(&def_range);
                let (used_node, linear) = range_to_node.get_use(&use_range);
                let new_node = self.create_normal_node(used_node, def_node, inst, linear);
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

    /// Create a normal node.
    fn create_normal_node(
        &mut self,
        used_node: Option<NodePtr>,
        def_node: Option<NodePtr>,
        inst: InstPtr,
        linear: bool,
    ) -> NodePtr {
        let node = self
            .builder
            .get_normal_node(used_node, def_node, inst, linear);
        self.inst_to_node.insert(inst, node);
        if let Some(used_node) = used_node {
            self.node_to_user.entry(used_node).or_default().insert(node);
        }
        if let Some(def_node) = def_node {
            self.node_to_user.entry(def_node).or_default().insert(node);
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
                    positions.push((bb, effect.def_range.clone()));
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

/// Memory pool for MemorySSA nodes.
struct MemorySSABuilder {
    node_pool: ObjPool<Node>,
    counter: usize,
}

impl MemorySSABuilder {
    /// Allocate a new node.
    fn new_node(&mut self, node: Node) -> NodePtr {
        self.node_pool.alloc(node)
    }

    /// Returns a unique ID.
    fn next_counter(&mut self) -> usize {
        let counter = self.counter;
        self.counter += 1;
        counter
    }

    /// Get an entry node.
    fn get_entry(&mut self) -> NodePtr {
        let next_counter = self.next_counter();
        self.new_node(Node::Entry(next_counter))
    }

    /// Get a normal node.
    fn get_normal_node(
        &mut self,
        used_node: Option<NodePtr>,
        def_node: Option<NodePtr>,
        inst: InstPtr,
        linear: bool,
    ) -> NodePtr {
        let next_counter = self.next_counter();
        self.new_node(Node::Normal(
            next_counter,
            used_node,
            def_node,
            inst,
            linear,
        ))
    }

    /// Get a phi node.
    fn get_phi(&mut self, range: EffectRange) -> NodePtr {
        let next_counter = self.next_counter();
        self.new_node(Node::Phi(next_counter, Vec::new(), range))
    }
}

/// Memory SSA node.
/// Function in Node does not maintain use-def chain.
pub enum Node {
    /// Entry(id) represents the memory state at the beginning of the function.
    Entry(usize),

    /// Normal(id, used_node, def_node, inst, linear) represents a memory state after an instruction.
    /// For a MemoryUse, if linear flag is true, it guarantees using used_node.
    Normal(usize, Option<NodePtr>, Option<NodePtr>, InstPtr, bool),

    /// Phi(id, args, range) represents a phi node.
    Phi(usize, Vec<(BBPtr, NodePtr)>, EffectRange),
}

impl Node {
    /// Get instruction if it's a normal node.
    pub fn get_inst(&self) -> Option<InstPtr> {
        match self {
            Node::Normal(_, _, _, inst, _) => Some(*inst),
            _ => None,
        }
    }

    /// Get ID of the node.
    pub fn get_id(&self) -> usize {
        match self {
            Node::Entry(id) => *id,
            Node::Normal(id, _, _, _, _) => *id,
            Node::Phi(id, _, _) => *id,
        }
    }

    /// Get used nodes.
    pub fn get_used_node(&self) -> Vec<NodePtr> {
        match self {
            Node::Normal(_, use_node, def_node, _, _) => {
                let mut result = Vec::new();
                if let Some(node) = use_node {
                    result.push(*node);
                }
                if let Some(node) = def_node {
                    result.push(*node);
                }
                result
            }
            Node::Phi(_, args, _) => args.iter().map(|(_, node)| *node).collect(),
            _ => Vec::new(),
        }
    }

    /// Check if node is memset.
    fn is_memset(&self) -> bool {
        self.get_inst()
            .map(|inst| {
                inst.get_type() == InstType::Call
                    && downcast_ref::<Call>(inst.as_ref().as_ref())
                        .func
                        .is_memset()
            })
            .unwrap_or(false)
    }

    /// Check if node is entry.
    fn is_entry(&self) -> bool {
        matches!(self, Node::Entry(_))
    }

    /// Add an argument to a phi node.
    fn add_phi_arg(&mut self, arg: (BBPtr, NodePtr)) {
        match self {
            Node::Phi(_, args, _) => args.push(arg),
            _ => panic!("not a phi node"),
        }
    }

    /// Get effect range of a phi node.
    fn get_effect_range(&self) -> &EffectRange {
        match self {
            Node::Phi(_, _, range) => range,
            _ => panic!("not a phi node"),
        }
    }

    /// Merge effect range of a phi node.
    fn merge_effect_range(&mut self, another: &EffectRange) {
        match self {
            Node::Phi(_, _, range) => range.merge(another),
            _ => panic!("not a phi node"),
        }
    }
}

/// Phi insertion for a block. (Some(Node) or None)
pub struct PhiInsertion(Option<NodePtr>);

impl PhiInsertion {
    /// Initialize an empty phi insertion.
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

/// Framed mapping from range to node.
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

    /// Get an element from all frames with def range.
    /// Returns the element.
    pub fn get_def(&self, def_range: &EffectRange) -> Option<NodePtr> {
        if def_range.is_empty() {
            return None;
        }
        let mut map = self;
        loop {
            match map {
                Self::Root(m) => return m.get_def(),
                Self::Leaf(m, parent) => {
                    if let Some(v) = m.get_def() {
                        return Some(v);
                    }
                    map = parent;
                }
            }
        }
    }

    /// Get an element from all frames with use range.
    /// Returns the element, and if the hit is exact hit.
    pub fn get_use(&self, use_range: &EffectRange) -> (Option<NodePtr>, bool) {
        if use_range.is_empty() {
            return (None, false);
        }
        let mut map = self;
        loop {
            match map {
                Self::Root(m) => return m.get_use(use_range),
                Self::Leaf(m, parent) => {
                    if let (Some(v), l) = m.get_use(use_range) {
                        return (Some(v), l);
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

/// One frame of range to node mapping.
#[derive(Default)]
pub struct RangeToNodeFrame(Vec<(EffectRange, NodePtr)>);

impl RangeToNodeFrame {
    pub fn insert(&mut self, k: EffectRange, v: NodePtr) {
        self.0.push((k, v));
    }

    /// Get the last definition.
    pub fn get_def(&self) -> Option<NodePtr> {
        self.0.last().map(|(_, n)| *n)
    }

    /// Get an element from the frame with given use range.
    /// Returns the element, and if the hit is predictable.
    ///
    /// Criterion for predictable is:
    /// - If effect range features only one variable, it's predictable.
    /// - If store node is memset, it's predictable because we assume memset makes everything zero.
    /// - If store node is entry, it's predictable so that load gvar in main function can reduce to constant.
    pub fn get_use(&self, use_range: &EffectRange) -> (Option<NodePtr>, bool) {
        for (key, value) in self.0.iter().rev() {
            if key.can_alias(use_range) {
                return (
                    Some(*value),
                    key == use_range || value.is_memset() || value.is_entry(),
                );
            }
        }
        (None, false)
    }
}
