use std::{collections::HashMap, hash::Hash};

pub enum FrameMap<'a, K, V> {
    Root(HashMap<K, V>),
    Leaf(HashMap<K, V>, &'a FrameMap<'a, K, V>),
}

impl<K, V> Default for FrameMap<'_, K, V> {
    fn default() -> Self {
        Self::Root(HashMap::new())
    }
}

impl<'a, K, V> FrameMap<'a, K, V>
where
    K: Eq + Hash,
{
    /// Create a new FrameMap.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the last frame.
    pub fn last_frame(&mut self) -> &mut HashMap<K, V> {
        match self {
            Self::Root(map) => map,
            Self::Leaf(map, _) => map,
        }
    }

    /// Insert a new element into the last frame.
    pub fn insert(&mut self, k: K, v: V) {
        self.last_frame().insert(k, v);
    }

    /// Get an element from all frames.
    pub fn get(&self, k: &K) -> Option<&V> {
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
        Self::Leaf(HashMap::new(), self)
    }
}
