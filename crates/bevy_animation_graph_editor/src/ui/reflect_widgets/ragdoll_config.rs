use bevy_animation_graph::core::ragdoll::configuration::RagdollConfig;

use crate::ui::{
    generic_widgets::ragdoll_config::RagdollConfigWidget,
    reflect_lib::{ReflectWidget, ReflectWidgetContext},
};

#[derive(Default)]
pub struct RagdollConfigReflectWidget;

impl ReflectWidget for RagdollConfigReflectWidget {
    type Target = RagdollConfig;

    fn draw(
        &self,
        ui: &mut egui::Ui,
        value: &mut Self::Target,
        _: &ReflectWidgetContext,
    ) -> egui::Response {
        ui.add(RagdollConfigWidget::new_salted(
            value,
            "ragdoll config widget",
        ))
    }
}
