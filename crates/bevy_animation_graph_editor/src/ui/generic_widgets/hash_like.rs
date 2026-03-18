use std::{hash::Hash, marker::PhantomData};

/// Creates a child Ui with a fixed width that won't overflow into the parent's
/// horizontal layout, even if the child content is wider than `col_width`.
fn fixed_width_column(
    ui: &mut egui::Ui,
    id: egui::Id,
    col_width: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> egui::Response,
) -> egui::Response {
    let col_rect = egui::Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(col_width, ui.max_rect().bottom() - ui.cursor().min.y),
    );
    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(id)
            .max_rect(col_rect)
            .layout(*ui.layout()),
    );
    let response = add_contents(&mut child_ui);
    let used_height = child_ui.min_rect().height();
    // Advance parent cursor by exactly col_width, regardless of child overflow
    ui.advance_cursor_after_rect(egui::Rect::from_min_size(
        col_rect.min,
        egui::vec2(col_width, used_height),
    ));
    response
}

pub struct HashLikeWidget<'a, K, V, C, H> {
    pub map: &'a mut H,
    pub id_hash: egui::Id,
    __k: PhantomData<K>,
    __v: PhantomData<V>,
    __c: PhantomData<C>,
}

impl<'a, K, V, H, C> HashLikeWidget<'a, K, V, C, H>
where
    K: Default + PartialOrd + Eq + Clone + Hash + Send + Sync + Ord + 'static,
    V: Clone + Send + Sync + 'static,
    C: Default + Clone + Send + Sync + 'static,
    H: HashLikeEditable<K, V, C>,
{
    pub fn new_salted(map: &'a mut H, salt: impl std::hash::Hash) -> Self {
        Self {
            map,
            id_hash: egui::Id::new(salt),
            __k: PhantomData,
            __v: PhantomData,
            __c: PhantomData,
        }
    }

    pub fn ui(self, ui: &mut egui::Ui) -> egui::Response
    where
        K: Default + PartialOrd + Eq + Clone + Hash + Send + Sync + 'static,
        V: Default + Clone,
    {
        ui.push_id(self.id_hash, |ui| {
            let mut response = ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());

            let mut buffer = HashLikeBuffer::from_ui_ordered(ui, self.map);
            let mut new_type_buffer = NewItemBuffer::from_ui(ui, self.map);
            let NewItemBuffer {
                key,
                value,
                context,
            } = &mut new_type_buffer;

            let mut pending_delete_key_idx = None;
            let mut pending_add_key = None;

            for (idx, key) in buffer.ordering.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.push_id(egui::Id::new(idx).with("delete button"), |ui| {
                        let button_response = ui.button("x");
                        if button_response.clicked() {
                            pending_delete_key_idx = Some(idx);
                            response.mark_changed();
                            self.map.delete_existing_key(key);
                        }
                        response |= button_response;
                    });

                    let spacing = ui.spacing().item_spacing.x;
                    let col_width = (ui.available_width() - spacing).max(0.) / 2.0;

                    response |= fixed_width_column(
                        ui,
                        egui::Id::new(idx).with("edit existing key"),
                        col_width,
                        |ui| self.map.edit_existing_key(ui, key),
                    );
                    response |= fixed_width_column(
                        ui,
                        egui::Id::new(idx).with("edit existing value"),
                        col_width,
                        |ui| self.map.edit_existing_value_for(ui, key),
                    );
                });
            }

            ui.separator();

            ui.horizontal(|ui| {
                ui.push_id(egui::Id::new(-1i32).with("create button"), |ui| {
                    let button_response = ui.button("+");
                    if button_response.clicked() {
                        pending_add_key = Some(key.clone());
                        response.mark_changed();
                        self.map
                            .add_new_value(key.clone(), value.clone(), context.clone());
                    }
                    response |= button_response;
                });

                let spacing = ui.spacing().item_spacing.x;
                let col_width = (ui.available_width() - spacing).max(0.) / 2.0;

                response |= fixed_width_column(
                    ui,
                    egui::Id::new(-1i32).with("edit new key"),
                    col_width,
                    |ui| self.map.edit_new_key(ui, key, context),
                );
                response |= fixed_width_column(
                    ui,
                    egui::Id::new(-1i32).with("edit new value"),
                    col_width,
                    |ui| self.map.edit_new_value(ui, value, context),
                );
            });

            if let Some(delete_idx) = pending_delete_key_idx {
                buffer.ordering.remove(delete_idx);
            }

            if let Some(add_key) = pending_add_key {
                buffer.ordering.push(add_key);
            }

            buffer.write_back(ui);
            new_type_buffer.write_back(ui);

            response
        })
        .inner
    }
}

#[derive(Clone)]
struct HashLikeBuffer<K> {
    ordering: Vec<K>,
}

impl<K: PartialEq + Clone + Send + Sync + 'static> HashLikeBuffer<K> {
    pub fn should_clean_up<V, C>(&self, hashlike: &impl HashLikeEditable<K, V, C>) -> bool {
        for key in &self.ordering {
            if hashlike.get(key).is_none() {
                return true;
            }
        }

        for key in hashlike.keys() {
            if !self.ordering.contains(key) {
                return true;
            }
        }

        false
    }

    pub fn from_ui_ordered<V, C>(
        ui: &mut egui::Ui,
        hashlike: &impl HashLikeEditable<K, V, C>,
    ) -> Self
    where
        K: Ord,
    {
        let buffer_id = Self::buffer_id(ui);

        ui.memory_mut(|mem| {
            if let Some(old_buffer) = mem.data.get_temp::<HashLikeBuffer<K>>(buffer_id)
                && !old_buffer.should_clean_up(hashlike)
            {
                return old_buffer;
            }

            let mut keys: Vec<K> = hashlike.keys().cloned().collect();

            keys.sort();

            HashLikeBuffer { ordering: keys }
        })
    }

    pub fn write_back(&self, ui: &mut egui::Ui) {
        let buffer_id = Self::buffer_id(ui);
        ui.memory_mut(|mem| mem.data.insert_temp(buffer_id, self.clone()));
    }

    pub fn buffer_id(ui: &egui::Ui) -> egui::Id {
        ui.id().with("hashlike widget buffer")
    }
}

#[derive(Clone)]
struct NewItemBuffer<K, V, C> {
    key: K,
    value: V,
    context: C,
}

impl<K, V, C> NewItemBuffer<K, V, C>
where
    K: Default + Clone + Send + Sync + 'static,
    V: Default + Clone + Send + Sync + 'static,
    C: Default + Clone + Send + Sync + 'static,
{
    pub fn should_clean_up(&self, _: &impl HashLikeEditable<K, V, C>) -> bool {
        false
    }

    pub fn from_ui(ui: &mut egui::Ui, hashlike: &impl HashLikeEditable<K, V, C>) -> Self
    where
        K: Ord,
    {
        let buffer_id = Self::buffer_id(ui);

        ui.memory_mut(|mem| {
            if let Some(old_buffer) = mem.data.get_temp::<Self>(buffer_id)
                && !old_buffer.should_clean_up(hashlike)
            {
                return old_buffer;
            }

            Self {
                key: K::default(),
                value: V::default(),
                context: C::default(),
            }
        })
    }

    pub fn write_back(&self, ui: &mut egui::Ui) {
        let buffer_id = Self::buffer_id(ui);
        ui.memory_mut(|mem| mem.data.insert_temp(buffer_id, self.clone()));
    }

    pub fn buffer_id(ui: &egui::Ui) -> egui::Id {
        ui.id().with("hashlike new item buffer")
    }
}

pub trait HashLikeEditable<K, V, C = ()> {
    fn get(&self, key: &K) -> Option<&V>;

    fn keys<'a>(&'a self) -> impl Iterator<Item = &'a K>
    where
        K: 'a;

    fn add_new_value(&mut self, key: K, value: V, context: C);
    fn delete_existing_key(&mut self, key: &K);

    fn edit_new_key(&mut self, ui: &mut egui::Ui, key: &mut K, context: &mut C) -> egui::Response;
    fn edit_new_value(
        &mut self,
        ui: &mut egui::Ui,
        value: &mut V,
        context: &mut C,
    ) -> egui::Response;

    /// Recommended that you update the key passed to you as well as update your hash like data
    /// structure. The widget won't do it for you
    fn edit_existing_key(&mut self, ui: &mut egui::Ui, key: &mut K) -> egui::Response;

    fn edit_existing_value_for(&mut self, ui: &mut egui::Ui, key: &K) -> egui::Response;
}
