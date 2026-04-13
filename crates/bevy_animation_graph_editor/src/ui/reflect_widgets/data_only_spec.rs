use std::marker::PhantomData;

use bevy::reflect::Reflect;
use bevy_animation_graph::core::context::spec_context::DataOnlySpec;

use crate::ui::{
    generic_widgets::data_only_spec::DataOnlySpecWidget,
    reflect_lib::{ReflectWidget, ReflectWidgetContext},
};

#[derive(Default)]
pub struct DataOnlySpecReflectWidget<K, V> {
    __k: PhantomData<K>,
    __v: PhantomData<V>,
}

impl<K, V> ReflectWidget for DataOnlySpecReflectWidget<K, V>
where
    K: Reflect + Default + Clone + std::hash::Hash + std::fmt::Debug + Ord + Send + Sync + 'static,
    V: Reflect + Default + PartialEq + Clone + Send + Sync + 'static,
{
    type Target = DataOnlySpec<K, V>;

    fn draw(
        &self,
        ui: &mut egui::Ui,
        value: &mut Self::Target,
        ctx: &ReflectWidgetContext,
    ) -> egui::Response {
        DataOnlySpecWidget::new(value)
            .salted("reflect data only spec widget")
            .show(ui, |ui, k| ctx.draw(ui, k), |ui, v| ctx.draw(ui, v))
    }
}
