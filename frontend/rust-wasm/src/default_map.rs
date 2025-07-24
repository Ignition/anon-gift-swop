use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct DefaultMap<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    pub(crate) map: HashMap<K, V>,
    pub(crate) default_value: V,
}

impl<K, V> DefaultMap<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    pub fn get(&self, key: &K) -> V {
        self.map
            .get(key)
            .cloned()
            .unwrap_or_else(|| self.default_value.clone())
    }
}
