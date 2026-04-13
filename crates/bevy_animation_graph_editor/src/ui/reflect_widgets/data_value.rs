use bevy_animation_graph::core::edge_data::DataValue;

use crate::ui::{
    generic_widgets::data_value::DataValueWidget,
    reflect_lib::{ReflectWidget, ReflectWidgetContext},
};

#[derive(Default)]
pub struct DataValueReflectWidget;

impl ReflectWidget for DataValueReflectWidget {
    type Target = DataValue;

    fn draw(
        &self,
        ui: &mut egui::Ui,
        value: &mut Self::Target,
        _: &ReflectWidgetContext,
    ) -> egui::Response {
        ui.add(DataValueWidget::new_salted(value, "ragdoll config widget"))
    }
}
