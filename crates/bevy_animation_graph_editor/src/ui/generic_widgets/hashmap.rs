use std::hash::Hash;

use bevy::platform::collections::HashMap;

use crate::ui::generic_widgets::hash_like::{HashLikeEditable, HashLikeWidget};

pub struct HashMapWidget<'a, K, V> {
    pub map: &'a mut HashMap<K, V>,
    pub id_hash: egui::Id,
}

impl<'a, K, V> HashMapWidget<'a, K, V> {
    pub fn new(map: &'a mut HashMap<K, V>) -> Self {
        Self {
            map,
            id_hash: egui::Id::new("hash map"),
        }
    }

    pub fn salted(mut self, salt: impl std::hash::Hash) -> Self {
        self.id_hash = egui::Id::new(salt);
        self
    }
}

impl<'a, K, V> HashMapWidget<'a, K, V>
where
    K: Default + PartialOrd + Eq + Clone + Hash + Send + Sync + Ord + 'static,
    V: Default + Send + Sync + Clone + 'static,
{
    pub fn ui(
        self,
        ui: &mut egui::Ui,
        edit_key: impl FnMut(&mut egui::Ui, &mut K) -> egui::Response,
        edit_value: impl FnMut(&mut egui::Ui, &mut V) -> egui::Response,
    ) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let response = HashLikeWidget::new_salted(
                &mut HashMapEditable {
                    map: self.map,
                    edit_key,
                    edit_value,
                },
                "hash like editable",
            )
            .ui(ui);

            response
        })
        .inner
    }
}

pub struct HashMapEditable<'a, K, V, F, G> {
    map: &'a mut HashMap<K, V>,
    edit_key: F,
    edit_value: G,
}

impl<'a, K, V, F, G> HashLikeEditable<K, V> for HashMapEditable<'a, K, V, F, G>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    F: FnMut(&mut egui::Ui, &mut K) -> egui::Response,
    G: FnMut(&mut egui::Ui, &mut V) -> egui::Response,
{
    fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    fn keys<'b>(&'b self) -> impl Iterator<Item = &'b K>
    where
        K: 'b,
    {
        self.map.keys()
    }

    fn add_new_value(&mut self, key: K, value: V, (): ()) {
        self.map.insert(key, value);
    }

    fn delete_existing_key(&mut self, key: &K) {
        self.map.remove(key);
    }

    fn edit_new_key(&mut self, ui: &mut egui::Ui, key: &mut K, (): &mut ()) -> egui::Response {
        (self.edit_key)(ui, key)
    }

    fn edit_new_value(&mut self, ui: &mut egui::Ui, value: &mut V, (): &mut ()) -> egui::Response {
        (self.edit_value)(ui, value)
    }

    fn edit_existing_key(&mut self, ui: &mut egui::Ui, key: &mut K) -> egui::Response {
        let mut buffer = KeyBuffer::from_ui(ui, key);
        let response = (self.edit_key)(ui, &mut buffer.buffer);
        buffer.write_back(ui);

        if response.changed() && !self.map.contains_key(&buffer.buffer) {
            *key = buffer.buffer.clone();
            if let Some(value) = self.map.remove(key) {
                self.map.insert(key.clone(), value);
            }
        }

        response
    }

    fn edit_existing_value_for(&mut self, ui: &mut egui::Ui, key: &K) -> egui::Response {
        if let Some(value) = self.map.get_mut(key) {
            (self.edit_value)(ui, value)
        } else {
            ui.label("key does not exist")
        }
    }
}

#[derive(Clone, Default)]
pub struct KeyBuffer<K> {
    buffer: K,
    original: K,
}

impl<K> KeyBuffer<K>
where
    K: PartialEq + Clone + Send + Sync + 'static,
{
    pub fn should_clean_up(&self, new_key: &K) -> bool {
        &self.original != new_key
    }

    pub fn from_ui(ui: &mut egui::Ui, new_key: &K) -> Self {
        let buffer_id = Self::buffer_id(ui);

        ui.memory_mut(|mem| {
            if let Some(old_buffer) = mem.data.get_temp::<Self>(buffer_id) {
                if !old_buffer.should_clean_up(new_key) {
                    return old_buffer;
                }
            }

            Self {
                buffer: new_key.clone(),
                original: new_key.clone(),
            }
        })
    }

    pub fn write_back(&self, ui: &mut egui::Ui) {
        let buffer_id = Self::buffer_id(ui);
        ui.memory_mut(|mem| mem.data.insert_temp(buffer_id, self.clone()));
    }

    pub fn buffer_id(ui: &egui::Ui) -> egui::Id {
        ui.id().with("hashmap existing key buffer")
    }
}
