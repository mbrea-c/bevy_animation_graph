use crate::core::animation_graph::EdgeSpec;
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

impl HashMapJoinExt<String, EdgeSpec> for HashMap<String, EdgeSpec> {
    type Val = EdgeSpec;

    fn fill_up<F>(&mut self, other: &HashMap<String, EdgeSpec>, mapper: &F) -> &mut Self
    where
        F: Fn(&EdgeSpec) -> Self::Val,
    {
        for (k, v) in other {
            if !self.contains_key(k) {
                self.insert(k.clone(), mapper(v));
            }
        }
        self
    }
}

impl<T> HashMapJoinExt<String, T> for HashMap<String, ()> {
    type Val = ();

    fn fill_up<F>(&mut self, other: &HashMap<String, T>, _: &F) -> &mut Self
    where
        F: Fn(&T) -> Self::Val,
    {
        for (k, _) in other {
            if !self.contains_key(k) {
                self.insert(k.clone(), ());
            }
        }
        self
    }
}
