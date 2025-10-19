use std::any::Any;

use bevy_animation_graph::core::ragdoll::configuration::RagdollConfig;
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui::Widget;
use egui_dock::egui;

use crate::ui::{
    generic_widgets::{
        bone_id::BoneIdWidget, hashmap::HashMapWidget, ragdoll_config::RagdollConfigWidget,
    },
    utils::with_assets_all,
};

use super::{EguiInspectorExtension, MakeBuffer};

#[derive(Default)]
pub struct RagdollConfigInspector;

impl EguiInspectorExtension for RagdollConfigInspector {
    type Base = RagdollConfig;
    type Buffer = ();

    fn mutable(
        value: &mut Self::Base,
        _buffer: &mut Self::Buffer,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        _id: egui::Id,
        _env: InspectorUi<'_, '_>,
    ) -> bool {
        RagdollConfigWidget::new_salted(value, "config widget")
            .ui(ui)
            .changed()
    }

    fn readonly(
        _value: &Self::Base,
        _buffer: &Self::Buffer,
        _ui: &mut egui::Ui,
        _options: &dyn Any,
        _id: egui::Id,
        _env: InspectorUi<'_, '_>,
    ) {
        todo!()
    }
}

impl MakeBuffer<()> for RagdollConfig {
    fn make_buffer(&self) {
        ()
    }
}
