#[macro_export]
macro_rules! define_graph_iterator {
    ($name:ident, $collection:ty, $pop_method:ident, $bb_update_method:ident) => {
        pub struct $name {
            container: $collection,
            visited: HashSet<BBPtr>,
        }

        impl Iterator for $name {
            type Item = BBPtr;
            fn next(&mut self) -> Option<Self::Item> {
                while let Some(bb) = self.container.$pop_method() {
                    if !self.visited.contains(&bb) {
                        self.visited.insert(bb);
                        self.container.extend(bb.$bb_update_method());
                        return Some(bb);
                    }
                }
                None
            }
        }

        impl From<BBPtr> for $name {
            fn from(bb: BBPtr) -> Self {
                Self {
                    container: vec![bb].into(),
                    visited: HashSet::new(),
                }
            }
        }
    };
}
