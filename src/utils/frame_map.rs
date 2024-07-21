use std::{collections::HashMap, hash::Hash};

pub struct FrameMap<K, V>(Vec<HashMap<K, V>>);

impl<K, V> Default for FrameMap<K, V> {
    fn default() -> Self {
        FrameMap(vec![HashMap::new()])
    }
}

impl<K, V> FrameMap<K, V>
where
    K: Eq + Hash,
{
    /// Insert a new element into the last frame.
    ///
    /// # Panics
    /// Please make sure the map contains at least one frame.
    pub fn insert(&mut self, k: K, v: V) {
        self.0.last_mut().unwrap().insert(k, v);
    }

    /// Get an element.
    pub fn get(&self, k: &K) -> Option<&V> {
        for frame in self.0.iter().rev() {
            if let Some(v) = frame.get(&k) {
                return Some(v);
            }
        }
        None
    }
}
