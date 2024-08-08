use std::collections::{HashMap, HashSet};

use crate::{
    middle::{
        analysis::dominator_tree::DominatorTree,
        ir::{BBPtr, FunPtr, InstPtr},
    },
    utils::mem::{ObjPool, ObjPtr},
};

use super::{alias_analysis::EffectRange, effect_analysis::EffectAnalysis};

pub struct MemorySSA {
    builder: MemorySSABuilder,
    inst_to_node: HashMap<InstPtr, NodePtr>,
    block_to_node: HashMap<BBPtr, NodePtr>,
    effect_analysis: EffectAnalysis,
}

#[allow(unused)]
impl MemorySSA {
    pub fn run(&mut self, func: FunPtr) {
        let entry = func.entry.unwrap();
        let phi_insertions = self.insert_empty_phi(func);

        // Add entry node
        let mut range_to_node = RangeToNode::new();
        let entry_node = self.builder.get_entry();
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
                let def_node = range_to_node.get(&effect.def_range.clone().into());
                let use_node = range_to_node.get(&effect.use_range.clone().into());
                self.create_normal_node(use_node, def_node, inst);
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
    ) {
        let node = self.builder.get_normal_node(use_node, def_node, inst);
        self.inst_to_node.insert(inst, node);
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

pub type NodePtr = ObjPtr<Node>;

pub struct MemorySSABuilder {
    node_pool: ObjPool<Node>,
}

impl MemorySSABuilder {
    pub fn new_node(&mut self, node: Node) -> NodePtr {
        self.node_pool.alloc(node)
    }

    pub fn get_entry(&mut self) -> NodePtr {
        self.new_node(Node::Entry)
    }

    pub fn get_normal_node(
        &mut self,
        use_node: Option<NodePtr>,
        def_node: Option<NodePtr>,
        inst: InstPtr,
    ) -> NodePtr {
        self.new_node(Node::Normal(use_node, def_node, inst))
    }

    pub fn get_phi(&mut self, range: EffectRange) -> NodePtr {
        self.new_node(Node::Phi(Vec::new(), range))
    }
}

type PhiArg = (BBPtr, NodePtr);

#[allow(unused)]
pub enum Node {
    Entry,
    Normal(Option<NodePtr>, Option<NodePtr>, InstPtr),
    Phi(Vec<PhiArg>, EffectRange),
}

impl Node {
    pub fn add_phi_arg(&mut self, arg: PhiArg) {
        match self {
            Node::Phi(args, _) => args.push(arg),
            _ => panic!("not a phi node"),
        }
    }

    pub fn get_effect_range(&self) -> &EffectRange {
        match self {
            Node::Phi(_, range) => range,
            _ => panic!("not a phi node"),
        }
    }

    pub fn merge_effect_range(&mut self, another: &EffectRange) {
        match self {
            Node::Phi(_, range) => range.merge(another),
            _ => panic!("not a phi node"),
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
