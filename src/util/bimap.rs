use std::hash::Hash;
use rustc_hash::FxHashMap;

#[derive(Default)]
pub struct BiMap<K: Hash + Clone + Eq, V: Hash + Clone + Eq> {
    forwards: FxHashMap<K, V>,
    backwards: FxHashMap<V, K>,
}

impl<K: Hash + Clone + Eq, V: Hash + Clone + Eq> BiMap<K,V> {
    pub fn new() -> Self {
        Self {
            forwards: FxHashMap::default(),
            backwards: FxHashMap::default(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<(K,V)> {
        let prev_value_opt = self.forwards.insert(key.to_owned(), value.to_owned());
        let prev_key_opt = self.backwards.insert(value, key);
        prev_key_opt.map(|prev_key| (prev_key, prev_value_opt.unwrap()))
    }

    pub fn get_value(&self, key: &K) -> Option<&V> {
        self.forwards.get(key)
    }

    pub fn get_key(&self, value: &V) -> Option<&K> {
        self.backwards.get(value)
    }

    pub fn remove_by_key(&mut self, key: &K) -> Option<(K, V)> {
        let Some(value) = self.forwards.remove(key) else {
            return None;
        };
        let key = self.backwards.remove(&value).unwrap();
        Some((key, value))
    }

    pub fn remove_by_value(&mut self, value: &V) -> Option<(K, V)> {
        let Some(key) = self.backwards.remove(value) else {
            return None;
        };
        let value = self.forwards.remove(&key).unwrap();
        Some((key, value))
    }
}