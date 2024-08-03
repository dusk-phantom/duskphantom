use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};

pub trait GraphNode: std::hash::Hash + std::cmp::Eq + Sized + Clone {}

impl<T> GraphNode for T where T: std::hash::Hash + std::cmp::Eq + Sized + Clone {}

pub trait GraphNodeFromStr: GraphNode {
    /// Parse the node from a str
    fn from_str(input: &str) -> Result<Self>
    where
        Self: Sized;
}

impl<T> GraphNodeFromStr for T
where
    T: std::str::FromStr + GraphNode,
{
    fn from_str(input: &str) -> Result<Self> {
        input.parse().map_err(|_| anyhow!("parse error"))
    }
}

/// Undirected graph
pub struct UdGraph<T: GraphNode> {
    id_alloc: u64,
    n_id: std::collections::HashMap<T, u64>,
    id_n: std::collections::HashMap<u64, T>,
    edges: std::collections::HashMap<u64, std::collections::HashSet<u64>>,
}

impl<T: GraphNode> UdGraph<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            id_alloc: 0,
            n_id: std::collections::HashMap::new(),
            id_n: std::collections::HashMap::new(),
            edges: std::collections::HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: T) -> u64 {
        if let Some(id) = self.n_id.get(&node) {
            return *id;
        }
        let id = self.id_alloc;
        assert!(id < u64::MAX);
        self.id_alloc += 1;
        self.n_id.insert(node.clone(), id);
        self.id_n.insert(id, node);

        self.edges.entry(id).or_default();

        id
    }

    /// if any node is not in the graph, it will be added to the graph.
    /// the self to self edge adding will be ignored
    pub fn add_edge(&mut self, from: T, to: T) {
        let from_id = self.add_node(from);
        let to_id = self.add_node(to);
        self._add_edge_by_id(from_id, to_id)
    }

    pub fn add_edge_ref(&mut self, from: &T, to: &T) {
        let from_id = self.add_node(from.clone());
        let to_id = self.add_node(to.clone());
        self._add_edge_by_id(from_id, to_id)
    }

    #[inline]
    fn _add_edge_by_id(&mut self, from_id: u64, to_id: u64) {
        if from_id == to_id {
            return;
        }
        self.edges.entry(from_id).or_default().insert(to_id);
        self.edges.entry(to_id).or_default().insert(from_id);
    }

    fn get_node(&self, id: u64) -> Option<&T> {
        self.id_n.get(&id)
    }

    pub fn nodes(&self) -> std::collections::hash_map::Values<u64, T> {
        self.id_n.values()
    }

    pub fn iter(&self) -> UdGraphIter<T> {
        UdGraphIter {
            graph: self,
            nodes_iter: self.id_n.keys(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.id_n.is_empty()
    }

    /// get neighbor nodes of the node `from`
    pub fn get_nbs<'a>(&'a self, from: &T) -> Option<Neighbors<'a, T>> {
        let id = self.n_id.get(from)?;
        let tos = self.edges.get(id)?;
        let tos_iter = tos.iter();
        Some(Neighbors {
            graph: self,
            tos,
            tos_iter,
        })
    }
}

pub struct UdGraphIter<'a, T: GraphNode> {
    graph: &'a UdGraph<T>,
    nodes_iter: std::collections::hash_map::Keys<'a, u64, T>,
}

impl<'a, T: GraphNode> Iterator for UdGraphIter<'a, T> {
    type Item = (&'a T, Neighbors<'a, T>);
    fn next(&mut self) -> Option<Self::Item> {
        self.nodes_iter.next().map(|id| {
            let node = self.graph.get_node(*id).unwrap();
            let nbs = self.graph.get_nbs(node).unwrap();
            (node, nbs)
        })
    }
}

pub struct Neighbors<'a, T: GraphNode> {
    graph: &'a UdGraph<T>,
    tos: &'a std::collections::HashSet<u64>,
    tos_iter: std::collections::hash_set::Iter<'a, u64>,
}
/// impl Iterator for ToNodes
impl<'a, T: GraphNode> Iterator for Neighbors<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.tos_iter
            .next()
            .map(|id| self.graph.get_node(*id).unwrap())
    }
}

impl<T: GraphNode> Neighbors<'_, T> {
    pub fn contains(&self, node: &T) -> bool {
        self.graph
            .n_id
            .get(node)
            .map_or(false, |id| self.tos.contains(id))
    }
}

impl<T: GraphNode> From<HashMap<T, HashSet<T>>> for UdGraph<T> {
    fn from(value: HashMap<T, HashSet<T>>) -> Self {
        let mut g = UdGraph::new();
        for (k, v) in value {
            for to in v {
                g.add_edge(k.clone(), to);
            }
        }
        g
    }
}
impl<T: GraphNode> From<HashSet<(T, T)>> for UdGraph<T> {
    fn from(value: HashSet<(T, T)>) -> Self {
        let mut g = UdGraph::new();
        for (k, v) in value {
            g.add_edge(k, v);
        }
        g
    }
}

impl<T: GraphNode> From<UdGraph<T>> for HashMap<T, HashSet<T>> {
    fn from(g: UdGraph<T>) -> Self {
        let mut res = HashMap::new();
        for (k, v) in g.edges.iter() {
            let k = g.get_node(*k).unwrap().clone();
            let mut vs = HashSet::new();
            for to in v {
                vs.insert(g.get_node(*to).unwrap().clone());
            }
            res.insert(k, vs);
        }
        res
    }
}

#[macro_export]
/// a macro to create a graph
/// # Example
/// ```rust
/// use graph::*;
/// let g: UdGraph<u32> = udgraph!(
///    {1 -> 2,3},
///   {2 -> 3}
/// ).unwrap();
/// ```
/// or
/// ```rust
/// use graph::*;
/// let g: UdGraph<u32> = udgraph!(u32; {1 -> 2,3}, {2 -> 3}).unwrap();
/// ```
macro_rules! udgraph {
    ($({$key:tt $sep:tt $($tos:tt),*}$(,)?)*) => {{
        let parse_graph=||->anyhow::Result<$crate::UdGraph<_>>{
            let mut g=$crate::UdGraph::new();
            $(
                $(
                    let k=$crate::GraphNodeFromStr::from_str(&stringify!($key))?;
                    let v=$crate::GraphNodeFromStr::from_str(&stringify!($tos))?;
                    g.add_edge(k,v);
                )*
            )*
            Ok(g)
        };
        parse_graph()
    }};
    ($n_ty:ty;$({$key:tt $sep:tt $($tos:tt),*}$(,)?)*) => {{
        let parse_graph=||->anyhow::Result<$crate::UdGraph<$n_ty>>{
            let mut g=$crate::UdGraph::new();
            $(
                $(
                    let k:$n_ty=$crate::GraphNodeFromStr::from_str(stringify!($key))?;
                    let v:$n_ty=$crate::GraphNodeFromStr::from_str(stringify!($tos))?;
                    g.add_edge(k,v);
                )*
            )*
            Ok(g)
        };
        parse_graph()
    }};
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn basic() {
        let mut g = UdGraph::<u32>::new();
        g.add_edge(1, 2);
        g.add_edge(1, 3);
        g.add_edge(2, 3);

        let mut ns = g.nodes().collect::<Vec<&u32>>();
        ns.sort();
        assert_eq!(ns, vec![&1, &2, &3]);

        let mut nbs: Vec<&u32> = g.get_nbs(&1).unwrap().collect();
        nbs.sort();
        assert_eq!(nbs, vec![&2, &3]);

        let mut nbs: Vec<&u32> = g.get_nbs(&2).unwrap().collect();
        nbs.sort();
        assert_eq!(nbs, vec![&1, &3]);
    }

    #[test]
    fn test_macro() {
        let g: UdGraph<u32> = udgraph!(
            {1 -> 2,3},
            {2 -> 3}
        )
        .unwrap();
        assert!(!g.is_empty());
        let hm: HashMap<u32, HashSet<u32>> = g.into();
        assert_eq!(hm.len(), 3);
        assert_eq!(hm.get(&1).unwrap().len(), 2);
        assert_eq!(hm.get(&2).unwrap().len(), 2);

        let g = udgraph!(u32; {1 -> 2,3}, {2 -> 3}).unwrap();
        assert!(!g.is_empty());
        let hm: HashMap<u32, HashSet<u32>> = g.into();
        assert_eq!(hm.len(), 3);
        assert_eq!(hm.get(&1).unwrap().len(), 2);
        assert_eq!(hm.get(&2).unwrap().len(), 2);
    }

    #[test]
    // the self to self edge adding will be ignored
    fn test_self_to_self() {
        let g: UdGraph<u32> = udgraph!({1 -> 1}).unwrap();
        assert!(!g.is_empty());
        let hm: HashMap<u32, HashSet<u32>> = g.into();
        assert_eq!(hm.len(), 1);
        assert_eq!(hm.get(&1).unwrap().len(), 0);
    }
}
