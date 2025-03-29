use std::any::Any;

use bevy_animation_graph::core::animation_clip::EntityPath;
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui_dock::egui;

use super::EguiInspectorExtension;

pub struct EntityPathInspector;

impl EguiInspectorExtension for EntityPathInspector {
    type Base = EntityPath;
    type Buffer = String;

    fn mutable(
        value: &mut Self::Base,
        buffer: Option<&mut Self::Buffer>,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        _id: egui::Id,
        _env: InspectorUi<'_, '_>,
    ) -> bool {
        let buffered = buffer.unwrap();
        let response = ui.text_edit_singleline(buffered);

        if response.lost_focus() {
            *value = EntityPath::from_slashed_string(buffered.clone());
            true
        } else if !response.has_focus() {
            *buffered = value.to_slashed_string();
            false
        } else {
            false
        }
    }

    fn readonly(
        value: &Self::Base,
        _buffer: Option<&Self::Buffer>,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        _id: egui::Id,
        _env: InspectorUi<'_, '_>,
    ) {
        let slashed_path = value.to_slashed_string();
        ui.label(slashed_path);
    }

    fn init_buffer(#[allow(unused_variables)] value: &Self::Base) -> Option<Self::Buffer> {
        Some(value.to_slashed_string())
    }

    fn needs_buffer() -> bool {
        true
    }
}
