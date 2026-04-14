use bevy_animation_graph::core::context::spec_context::DataOnlySpec;

use crate::ui::{
    generic_widgets::list_like::{ListLike, ListLikeWidget},
    utils::ui_buffer::{CloneBuffer, SelfContainedBuffer},
};

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
    K: Default + Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Default + Clone + PartialEq + Send + Sync + 'static,
{
    pub fn show(
        self,
        ui: &mut egui::Ui,
        show_k: impl Fn(&mut egui::Ui, &mut K) -> egui::Response,
        show_v: impl Fn(&mut egui::Ui, &mut V) -> egui::Response,
    ) -> egui::Response {
        ListLikeWidget::new(&mut ValueWrapper {
            value: self.value,
            show_k,
            show_v,
        })
        .salted(self.id_hash)
        .show(ui)
    }
}

struct ValueWrapper<'a, K, V, F, G> {
    value: &'a mut DataOnlySpec<K, V>,
    show_k: F,
    show_v: G,
}

impl<'a, K, V, F, G> ListLike for ValueWrapper<'a, K, V, F, G>
where
    K: Default + Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Default + Clone + PartialEq + Send + Sync + 'static,
    F: Fn(&mut egui::Ui, &mut K) -> egui::Response,
    G: Fn(&mut egui::Ui, &mut V) -> egui::Response,
{
    type Item = (K, V);
    type ItemBuffer = EntryBuffer<K, V>;

    fn iter(&self) -> impl Iterator<Item = &Self::Item> {
        self.value.iter()
    }

    fn len(&self) -> usize {
        self.value.len()
    }

    fn shift_index(&mut self, index: usize, delta: i32) -> usize {
        self.value.shift_index(index, delta)
    }

    fn remove(&mut self, index: usize) {
        self.value.remove_index(index);
    }

    fn update(&mut self, index: usize, buffer: &Self::ItemBuffer) -> bool {
        let (key, value) = *buffer.value();
        self.value.update_index(index, key, value)
    }

    fn push(&mut self, buffer: &Self::ItemBuffer) {
        let (key, value) = *buffer.value();
        self.value.push(key, value);
    }

    fn default(&self) -> Option<Box<Self::Item>> {
        Some(Box::new((K::default(), V::default())))
    }

    fn edit_item(&self, ui: &mut egui::Ui, buffer: &mut Self::ItemBuffer) -> egui::Response {
        ui.vertical(|ui| {
            let mut response = (self.show_k)(ui, &mut buffer.key);
            response |= (self.show_v)(ui, &mut buffer.value);
            response
        })
        .inner
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct EntryBuffer<K, V> {
    key: K,
    value: V,

    original_key: K,
    original_value: V,
}

impl<K, V> CloneBuffer<(K, V), ()> for EntryBuffer<K, V>
where
    K: Clone + PartialEq + Send + Sync + 'static,
    V: Clone + PartialEq + Send + Sync + 'static,
{
    fn new(_: &egui::Ui, (): &(), (key, value): &(K, V)) -> Self {
        Self {
            key: key.clone(),
            value: value.clone(),
            original_key: key.clone(),
            original_value: value.clone(),
        }
    }

    fn id(&self, ui: &egui::Ui) -> egui::Id {
        ui.id().with("item")
    }

    fn is_still_valid(&self, (): &(), (key, value): &(K, V)) -> bool {
        &self.original_key == key && &self.original_value == value
    }
}

impl<K, V> SelfContainedBuffer<(K, V), ()> for EntryBuffer<K, V>
where
    K: Clone + PartialEq + Send + Sync + 'static,
    V: Clone + PartialEq + Send + Sync + 'static,
{
    fn value(&self) -> Box<(K, V)> {
        Box::new((self.key.clone(), self.value.clone()))
    }
}
