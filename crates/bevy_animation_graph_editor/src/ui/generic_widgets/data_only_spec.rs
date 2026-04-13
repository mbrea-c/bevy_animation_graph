use bevy_animation_graph::core::context::spec_context::DataOnlySpec;

pub struct DataOnlySpecWidget<'a, K, V> {
    pub value: &'a mut DataOnlySpec<K, V>,
    pub id_hash: egui::Id,
}

impl<'a, K, V> DataOnlySpecWidget<'a, K, V> {
    pub fn new(io_spec: &'a mut DataOnlySpec<K, V>) -> Self {
        Self {
            value: io_spec,
            id_hash: egui::Id::new("data only io spec"),
        }
    }

    pub fn salted(mut self, salt: impl std::hash::Hash) -> Self {
        self.id_hash = self.id_hash.with(salt);
        self
    }
}

impl<'a, K, V> DataOnlySpecWidget<'a, K, V>
where
    K: Clone + std::fmt::Debug + Eq + std::hash::Hash + Default + Send + Sync + 'static,
    V: Clone + PartialEq + Default + Send + Sync + 'static,
{
    pub fn show(
        mut self,
        ui: &mut egui::Ui,
        show_k: impl Fn(&mut egui::Ui, &mut K) -> egui::Response,
        show_v: impl Fn(&mut egui::Ui, &mut V) -> egui::Response,
    ) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            ui.vertical(|ui| {
                let response = egui::Frame::new()
                    .outer_margin(3.)
                    .inner_margin(3.)
                    .corner_radius(3.)
                    .stroke((1., ui.style().visuals.weak_text_color()))
                    .show(ui, |ui| {
                        let mut response = ui.heading("Inputs");

                        let kv: Vec<_> = self
                            .value
                            .sorted()
                            .into_iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();

                        for (i, (k, v)) in kv.into_iter().enumerate() {
                            ui.push_id(i, |ui| {
                                response |= self.show_field(ui, &show_k, &show_v, &k, &v, i);
                            });
                        }

                        ui.horizontal(|ui| {
                            if ui.button("+").clicked() {
                                self.value.add_data(K::default(), V::default());
                            }
                            ui.label("Add item");
                        });

                        response
                    })
                    .inner;

                response
            })
            .inner
        })
        .inner
    }

    fn show_field(
        &mut self,
        ui: &mut egui::Ui,
        show_k: impl Fn(&mut egui::Ui, &mut K) -> egui::Response,
        show_v: impl Fn(&mut egui::Ui, &mut V) -> egui::Response,
        key: &K,
        value: &V,
        index: usize,
    ) -> egui::Response {
        let mut buffer = ItemBuffer::from_ui(ui, index, key, value);

        ui.horizontal(|ui| {
            let mut response = self.item_controls(
                ui,
                index,
                self.value.len(),
                |this| {
                    this.value.shift_index(key, -1);
                },
                |this| {
                    this.value.shift_index(key, 1);
                },
                |this| {
                    this.value.remove(key);
                },
            );

            ui.vertical(|ui| {
                response |= show_k(ui, &mut buffer.key);
                response |= show_v(ui, &mut buffer.value);
            });

            buffer.save_back(ui);

            if response.changed()
                && self
                    .value
                    .update(&key, buffer.key.clone(), buffer.value.clone())
            {
                buffer.clear(ui);
            }

            response
        })
        .inner
    }

    fn item_controls(
        &mut self,
        ui: &mut egui::Ui,
        i: usize,
        size: usize,
        move_up_callback: impl FnOnce(&mut Self),
        move_down_callback: impl FnOnce(&mut Self),
        delete_callback: impl FnOnce(&mut Self),
    ) -> egui::Response {
        ui.scope(|ui| {
            ui.set_min_width(60.);
            let mut move_up = None;
            let mut move_down = None;
            let mut delete = None;

            let button =
                |ui: &mut egui::Ui, text: &str| ui.add(egui::Button::new(text).frame(false));

            let mut response = button(ui, "🗙");
            if response.clicked() {
                delete = Some(i);
            }

            let up_response = ui.add_enabled_ui(i > 0, |ui| button(ui, "⬆")).inner;
            if i > 0 && up_response.clicked() {
                move_up = Some(i);
            }
            response |= up_response;

            let down_response = ui
                .add_enabled_ui(i < (size - 1), |ui| button(ui, "⬇"))
                .inner;
            if i < size - 1 && down_response.clicked() {
                move_down = Some(i);
            }
            response |= down_response;

            if move_up.is_some() {
                response.mark_changed();
                move_up_callback(self);
            }

            if move_down.is_some() {
                response.mark_changed();
                move_down_callback(self);
            }

            if delete.is_some() {
                response.mark_changed();
                delete_callback(self);
            }

            response
        })
        .inner
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ItemBuffer<K, V> {
    key: K,
    value: V,

    id: egui::Id,
    original_key: K,
    original_value: V,
}

impl<K: Default, V: Default> Default for ItemBuffer<K, V> {
    fn default() -> Self {
        Self {
            key: Default::default(),
            value: Default::default(),
            id: egui::Id::new(0),
            original_key: Default::default(),
            original_value: Default::default(),
        }
    }
}

impl<K, V> ItemBuffer<K, V>
where
    K: Clone + PartialEq + Default + Send + Sync + 'static,
    V: Clone + PartialEq + Default + Send + Sync + 'static,
{
    pub fn from_ui(ui: &mut egui::Ui, idx: usize, key: &K, value: &V) -> Self {
        let id = Self::id(ui, idx);

        let buffer = ui.memory_mut(|mem| {
            mem.data
                .get_temp_mut_or_insert_with(id, || Self::new(ui, idx, key, value))
                .clone()
        });

        if &buffer.original_key == key && &buffer.original_value == value {
            buffer
        } else {
            Self::new(ui, idx, key, value)
        }
    }

    fn new(ui: &egui::Ui, idx: usize, key: &K, value: &V) -> Self {
        let id = Self::id(ui, idx);
        Self {
            key: key.clone(),
            value: value.clone(),
            id,
            original_key: key.clone(),
            original_value: value.clone(),
        }
    }

    pub fn id(ui: &egui::Ui, idx: usize) -> egui::Id {
        ui.id().with("item").with(idx)
    }

    pub fn save_back(&self, ui: &mut egui::Ui) {
        ui.memory_mut(|mem| {
            mem.data.insert_temp(self.id, self.clone());
        });
    }

    pub fn clear(&self, ui: &mut egui::Ui) {
        ui.memory_mut(|mem| {
            mem.data.remove_temp::<Self>(self.id);
        });
    }
}
