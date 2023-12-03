use bevy::reflect::Reflect;
use bevy::utils::HashMap;

#[derive(Clone, Copy, Reflect, Debug)]
pub enum InterpolationMode {
    Constant,
    Linear,
}

pub trait HashMapJoinExt<K, V> {
    type Val;

    fn fill_up<F>(&mut self, other: &HashMap<K, V>, mapper: &F) -> &mut Self
    where
        F: Fn(&V) -> Self::Val;
}

impl<T: Clone> HashMapJoinExt<String, T> for HashMap<String, T> {
    type Val = T;

    fn fill_up<F>(&mut self, other: &HashMap<String, T>, mapper: &F) -> &mut Self
    where
        F: Fn(&T) -> Self::Val,
    {
        for (k, v) in other {
            if !self.contains_key(k) {
                self.insert(k.clone(), mapper(v));
            }
        }
        self
    }
}

pub trait HashMapOpsExt<V> {
    fn extend_if_missing_owned(&mut self, other: HashMap<String, V>) -> &mut Self;
    fn extend_replacing_owned(&mut self, other: HashMap<String, V>) -> &mut Self;
}

impl<V: Clone> HashMapOpsExt<V> for HashMap<String, V> {
    fn extend_if_missing_owned(&mut self, mut other: HashMap<String, V>) -> &mut Self {
        for (k, v) in other.drain() {
            if !self.contains_key(&k) {
                self.insert(k, v);
            }
        }
        self
    }

    fn extend_replacing_owned(&mut self, mut other: HashMap<String, V>) -> &mut Self {
        for (k, v) in other.drain() {
            self.insert(k, v);
        }
        self
    }
}
