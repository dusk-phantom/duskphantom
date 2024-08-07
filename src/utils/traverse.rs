use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};

pub trait Node: Eq + Hash + Clone {
    fn get_succ(&self) -> Vec<Self>;
}

/// Postorder iterator.
pub struct POIterator<T>
where
    T: Node,
{
    container: VecDeque<T>,
}

impl<T> Iterator for POIterator<T>
where
    T: Node,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.container.pop_front()
    }
}

impl<T> From<T> for POIterator<T>
where
    T: Node,
{
    fn from(bb: T) -> Self {
        // Run postorder traversal
        let mut container = Vec::new();
        let mut visited = HashSet::new();
        run_postorder(bb, &mut visited, &mut container);

        // Wrap in iterator
        Self {
            container: container.into(),
        }
    }
}

/// Reverse postorder iterator.
pub struct RPOIterator<T>
where
    T: Node,
{
    container: Vec<T>,
}

impl<T> Iterator for RPOIterator<T>
where
    T: Node,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.container.pop()
    }
}

impl<T> From<T> for RPOIterator<T>
where
    T: Node,
{
    fn from(bb: T) -> Self {
        // Run postorder traversal
        let mut container = Vec::new();
        let mut visited = HashSet::new();
        run_postorder(bb, &mut visited, &mut container);

        // Wrap in iterator
        Self { container }
    }
}

/// Run a complete post order traversal.
fn run_postorder<T>(bb: T, visited: &mut HashSet<T>, container: &mut Vec<T>)
where
    T: Node,
{
    if visited.contains(&bb) {
        return;
    }
    visited.insert(bb.clone());
    for succ in bb.get_succ() {
        run_postorder(succ.clone(), visited, container);
    }
    container.push(bb);
}
