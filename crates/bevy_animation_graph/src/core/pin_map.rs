use bevy::{reflect::Reflect, utils::HashMap};

use super::animation_graph::PinId;

pub type PinMap<V> = HashMap<PinId, V>;

#[derive(Clone, Copy, Default, Debug, Reflect)]
pub enum GroupKey {
    #[default]
    Ungrouped,
    Group(i32),
}

#[derive(Debug, Clone, Reflect)]
pub struct SpecMap<V> {
    values: HashMap<PinId, V>,
    /// Determines the order to display the pins in the UI. Note that the order is "shared"
    /// between all input and all output pins, respectively (e.g. input data and time pins can be
    /// interleaved).
    order_key: HashMap<PinId, i32>,
    /// Pins adjacent in the order that share a group key will be displayed together
    group_key: HashMap<PinId, GroupKey>,
}

impl<V> Default for SpecMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> SpecMap<V> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            order_key: HashMap::new(),
            group_key: HashMap::new(),
        }
    }

    pub fn new_from_iter<I, K, T>(values: I) -> Self
    where
        I: IntoIterator<Item = (K, T)>,
        K: Into<PinId>,
        T: Into<V>,
    {
        let mut values_map = HashMap::new();
        let mut order_key = HashMap::new();

        for (i, (k, v)) in values.into_iter().enumerate() {
            let key = k.into();
            values_map.insert(key.clone(), v.into());
            order_key.insert(key, i as i32);
        }

        Self {
            values: values_map,
            order_key,
            group_key: HashMap::new(),
        }
    }

    pub fn add(&mut self, id: PinId, value: V, order: i32, group: GroupKey) {
        self.values.insert(id.clone(), value);
        self.order_key.insert(id.clone(), order);
        self.group_key.insert(id, group);
    }

    pub fn add_value(&mut self, id: PinId, value: V) {
        self.add(id, value, 0, GroupKey::Ungrouped);
    }

    pub fn with(mut self, id: PinId, value: V, order: i32, group: GroupKey) -> Self {
        self.add(id, value, order, group);
        self
    }

    pub fn keys(&self) -> impl Iterator<Item = &PinId> {
        self.values.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.values.values()
    }

    pub fn get(&self, id: &PinId) -> Option<&V> {
        self.values.get(id)
    }
}

impl<V, T: IntoIterator<Item = (PinId, V)>> From<T> for SpecMap<V> {
    fn from(value: T) -> Self {
        Self::new_from_iter(value)
    }
}
