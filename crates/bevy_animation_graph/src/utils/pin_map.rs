use std::hash::Hash;

use crate::core::animation_graph::PinId;
use bevy::{reflect::Reflect, utils::HashMap};
use serde::{Deserialize, Serialize};

pub type PinMap<V> = OrderedMap<PinId, V>;

#[derive(Debug, Clone, Reflect)]
pub struct OrderedMap<K, V> {
    values: HashMap<K, V>,
    index: Vec<K>,
    reverse_index: HashMap<K, usize>,
}

impl<K: Eq + Hash + Clone, V: Clone> Default for OrderedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash + Clone, V: Clone> OrderedMap<K, V> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            index: Vec::new(),
            reverse_index: HashMap::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.values.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.values.get_mut(key)
    }

    pub fn index(&self, key: &K) -> Option<usize> {
        self.reverse_index.get(key).copied()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if !self.reverse_index.contains_key(&key) {
            self.index.push(key.clone());
            self.reverse_index.insert(key.clone(), self.index.len());
        }

        self.values.insert(key, value)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.into_iter()
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.into_iter().map(|(k, _)| k)
    }
}

impl<K: Eq + Hash + Clone, V> FromIterator<(K, V)> for OrderedMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut values = HashMap::new();
        let mut index = Vec::new();
        let mut reverse_index = HashMap::new();

        for (i, (k, v)) in iter.into_iter().enumerate() {
            values.insert(k.clone(), v);
            index.push(k.clone());
            reverse_index.insert(k, i);
        }

        OrderedMap {
            values,
            index,
            reverse_index,
        }
    }
}

impl<K, V, FK, FV, const N: usize> From<[(FK, FV); N]> for OrderedMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
    FK: Into<K>,
    FV: Into<V>,
{
    fn from(value: [(FK, FV); N]) -> Self {
        let mut values = HashMap::new();
        let mut index = Vec::new();
        let mut reverse_index = HashMap::new();

        for (i, (k, v)) in value.into_iter().enumerate() {
            let k = k.into();
            let v = v.into();

            values.insert(k.clone(), v);
            index.push(k.clone());
            reverse_index.insert(k, i);
        }

        OrderedMap {
            values,
            index,
            reverse_index,
        }
    }
}

impl<K: Eq + Hash, V> IntoIterator for OrderedMap<K, V> {
    type Item = (K, V);
    type IntoIter = <Vec<(K, V)> as IntoIterator>::IntoIter;

    fn into_iter(mut self) -> Self::IntoIter {
        self.index
            .into_iter()
            .map(|k| {
                let val = self.values.remove(&k).unwrap();
                (k, val)
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<'a, K: Eq + Hash, V> IntoIterator for &'a OrderedMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = <Vec<(&'a K, &'a V)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.index
            .iter()
            .map(|k| {
                let val = self.values.get(k).unwrap();
                (k, val)
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<K: Eq + Hash + Clone + Serialize, V: Clone + Serialize> Serialize for OrderedMap<K, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.iter().collect::<Vec<_>>().serialize(serializer)
    }
}

impl<'a, K: Eq + Hash + Clone + Deserialize<'a>, V: Clone + Deserialize<'a>> Deserialize<'a>
    for OrderedMap<K, V>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        Ok(Self::from_iter(Vec::deserialize(deserializer)?))
    }
}
