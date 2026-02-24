use std::marker::PhantomData;

use bevy::{platform::collections::HashMap, reflect::Reflect};

use crate::ui::{
    generic_widgets::hashmap::HashMapWidget,
    reflect_lib::{ReflectWidget, ReflectWidgetContext},
};

#[derive(Default)]
pub struct HashMapReflectWidget<K, V> {
    __k: PhantomData<K>,
    __v: PhantomData<V>,
}

impl<K, V> ReflectWidget for HashMapReflectWidget<K, V>
where
    K: Reflect + Default + Clone + std::hash::Hash + Ord + Send + Sync + 'static,
    V: Reflect + Default + Clone + Send + Sync + 'static,
{
    type Target = HashMap<K, V>;

    fn draw(
        &self,
        ui: &mut egui::Ui,
        value: &mut Self::Target,
        ctx: &ReflectWidgetContext,
    ) -> egui::Response {
        HashMapWidget::new(value)
            .salted("reflect hashmap widget")
            .ui(ui, |ui, k| ctx.draw(ui, k), |ui, v| ctx.draw(ui, v))
    }
}
