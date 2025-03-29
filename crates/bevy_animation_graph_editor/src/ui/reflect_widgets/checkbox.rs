use std::any::Any;

use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui_dock::egui;

use super::EguiInspectorExtension;

#[derive(Default)]
pub struct CheckboxInspector;

impl EguiInspectorExtension for CheckboxInspector {
    type Base = bool;
    type Buffer = ();

    fn mutable(
        value: &mut Self::Base,
        _buffer: Option<&mut Self::Buffer>,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        _id: egui::Id,
        _env: InspectorUi<'_, '_>,
    ) -> bool {
        ui.checkbox(value, "").changed
    }

    fn readonly(
        value: &Self::Base,
        _buffer: Option<&Self::Buffer>,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        _id: egui::Id,
        _env: InspectorUi<'_, '_>,
    ) {
        let mut val = *value;
        ui.add_enabled_ui(false, |ui| ui.checkbox(&mut val, ""));
    }

    fn init_buffer(#[allow(unused_variables)] value: &Self::Base) -> Option<Self::Buffer> {
        None
    }

    fn needs_buffer() -> bool {
        false
    }
}
