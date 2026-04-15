use bevy::{platform::collections::HashMap, reflect::Reflect};
use serde::{Deserialize, Serialize};

#[derive(Reflect, Debug, Clone)]
pub struct SortedMap<K, V> {
    entries: Vec<(K, V)>,
    index: HashMap<K, usize>,
}

impl<K, V> Default for SortedMap<K, V> {
    fn default() -> Self {
        Self {
            entries: Default::default(),
            index: Default::default(),
        }
    }
}

impl<K, V> SortedMap<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    pub fn push(&mut self, key: K, value: V) -> bool {
        if self.index.contains_key(&key) {
            return false;
        }
        let entry = (key.clone(), value);
        self.entries.push(entry);
        self.index.insert(key, self.entries.len() - 1);
        true
    }

    pub fn input_compare_key(&self, key: &K) -> i32 {
        self.index.get(key).copied().map_or(i32::MAX, |i| i as i32)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.index.get(key).map(|i| &self.entries[*i].1)
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.entries.iter().map(|entry| &entry.0)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.entries.iter().map(|entry| &entry.1)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn shift_key(&mut self, key: &K, idx_delta: i32)
    where
        K: Clone + Eq + std::hash::Hash,
    {
        let Some(&current_idx) = self.index.get(key) else {
            return;
        };

        self.shift_index(current_idx, idx_delta);
    }

    pub fn shift_index(&mut self, current_idx: usize, idx_delta: i32) -> usize {
        let mut target_idx = (current_idx as i32 + idx_delta) as usize;
        if target_idx > current_idx {
            target_idx -= 1;
        }

        let current_entry = self.entries.remove(current_idx);
        self.entries.insert(target_idx, current_entry);

        self.reindex();

        target_idx
    }

    /// Returns false if the update could not be completed (e.g. if the new key already
    /// exists!)
    pub fn update(&mut self, prev_key: &K, new_key: K, new_value: V) -> bool
    where
        K: Clone + Eq + std::hash::Hash,
    {
        let Some(&index) = self.index.get(prev_key) else {
            return false;
        };

        self.update_index(index, new_key, new_value)
    }

    /// Returns false if the update could not be completed (e.g. if the new key already
    /// exists!)
    pub fn update_index(&mut self, index: usize, new_key: K, new_value: V) -> bool
    where
        K: Clone + Eq + std::hash::Hash,
    {
        let prev_key = &self.entries[index].0;

        if &new_key != prev_key && self.index.contains_key(&new_key) {
            return false;
        }

        self.index.remove(prev_key);
        self.entries.remove(index);
        self.entries.insert(index, (new_key.clone(), new_value));
        self.index.insert(new_key, index);

        true
    }

    pub fn remove_key(&mut self, key: &K)
    where
        K: Clone + Eq + std::hash::Hash,
    {
        if let Some(i) = self.index.remove(key) {
            self.entries.remove(i);
        }

        self.reindex();
    }

    pub fn remove_index(&mut self, index: usize) {
        self.entries.remove(index);
        self.reindex();
    }

    fn reindex(&mut self) {
        self.index.clear();
        self.entries.iter().enumerate().for_each(|(i, (k, _))| {
            self.index.insert(k.clone(), i);
        });
    }
}

impl<I, V> Serialize for SortedMap<I, V>
where
    I: Clone + Serialize + Eq + std::hash::Hash,
    V: Clone + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SortedMapSerial {
            entries: self.entries.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de, K, V> Deserialize<'de> for SortedMap<K, V>
where
    K: Deserialize<'de> + Eq + std::hash::Hash + Clone,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let serial = SortedMapSerial::deserialize(deserializer)?;

        let mut val = Self {
            entries: serial.entries,
            index: HashMap::new(),
        };

        val.reindex();

        Ok(val)
    }
}

#[derive(Serialize, Deserialize)]
struct SortedMapSerial<K: Eq + std::hash::Hash, V> {
    entries: Vec<(K, V)>,
}
