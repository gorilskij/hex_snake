use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};
use std::hash::Hash;

#[derive(Deref, DerefMut, Clone, Debug)]
pub struct HashMapWithDefault<K, V> {
    #[deref]
    #[deref_mut]
    map: HashMap<K, V>,
    default: V,
}

impl<K: Hash + Eq, V> Index<&K> for HashMapWithDefault<K, V> {
    type Output = V;

    fn index(&self, index: &K) -> &Self::Output {
        self.map.get(index).unwrap_or(&self.default)
    }
}

// NOTE: this allows modification of the default value after creation
impl<K: Hash + Eq, V> IndexMut<&K> for HashMapWithDefault<K, V> {
    fn index_mut(&mut self, index: &K) -> &mut Self::Output {
        self.map.get_mut(index).unwrap_or(&mut self.default)
    }
}

impl<K: Hash + Eq, V> HashMapWithDefault<K, V> {
    pub fn new(default: V) -> Self {
        Self { map: HashMap::new(), default }
    }

    pub fn get(&self, key: &K) -> &V {
        &self[key]
    }

    pub fn get_mut(&mut self, key: &K) -> &mut V {
        &mut self[key]
    }
}
