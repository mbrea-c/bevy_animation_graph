use std::{cmp::Ordering, hash::Hash};

use bevy::platform::collections::HashMap;

pub struct HashMapWidget<'a, K, V> {
    pub map: &'a mut HashMap<K, V>,
    pub id_hash: egui::Id,
}

impl<'a, K, V> HashMapWidget<'a, K, V> {
    pub fn new_salted(map: &'a mut HashMap<K, V>, salt: impl std::hash::Hash) -> Self {
        Self {
            map,
            id_hash: egui::Id::new(salt),
        }
    }
}

impl<'a, K, V> HashMapWidget<'a, K, V> {
    pub fn ui(
        self,
        ui: &mut egui::Ui,
        mut edit_key: impl FnMut(&mut egui::Ui, &mut K) -> egui::Response,
        mut show_key: impl FnMut(&mut egui::Ui, &K) -> egui::Response,
        mut edit_value: impl FnMut(&mut egui::Ui, &mut V) -> egui::Response,
    ) -> egui::Response
    where
        K: Default + PartialOrd + Eq + Clone + Hash + Send + Sync + 'static,
        V: Default,
    {
        ui.push_id(self.id_hash, |ui| {
            let mut response = ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());

            let vertical_response = ui.vertical(|ui| {
                let mut delete = None;

                let mut keys: Vec<_> = self.map.keys().cloned().collect();
                keys.sort_by(|l, r| l.partial_cmp(r).unwrap_or(Ordering::Equal));

                egui::Grid::new(self.id_hash.with("hashmap grid")).show(ui, |ui| {
                    for key in keys {
                        let entry = self.map.entry(key.clone());
                        ui.push_id(key.clone(), |ui| {
                            ui.horizontal(|ui| {
                                if ui.button("🗙").clicked() {
                                    delete = Some(key.clone());
                                    response.mark_changed();
                                }
                                response |= show_key(ui, entry.key());
                            });
                        });

                        ui.push_id(key, |ui| {
                            response |= edit_value(ui, entry.or_insert_with(|| V::default()));
                        });

                        ui.end_row();
                    }

                    let key_cache_id = ui.id().with("cache key");

                    let mut key_cache = ui.memory_mut(|mem| {
                        mem.data.get_temp_mut_or_default::<K>(key_cache_id).clone()
                    });

                    ui.horizontal(|ui| {
                        if ui.button("+").clicked() {
                            self.map.insert(key_cache.clone(), V::default());
                            response.mark_changed();
                        }

                        edit_key(ui, &mut key_cache);
                    });

                    ui.memory_mut(|mem| mem.data.insert_temp(key_cache_id, key_cache));

                    ui.label("<New item>");
                    ui.end_row();
                });

                if let Some(key) = delete {
                    self.map.remove(&key);
                }
            });

            response |= vertical_response.response;

            response
        })
        .inner
    }
}
