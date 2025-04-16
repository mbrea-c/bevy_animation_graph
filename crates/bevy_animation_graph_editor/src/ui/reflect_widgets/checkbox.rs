use std::any::Any;

use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui_dock::egui;

use super::{EguiInspectorExtension, MakeBuffer};

#[derive(Default)]
pub struct CheckboxInspector;

impl EguiInspectorExtension for CheckboxInspector {
    type Base = bool;
    type Buffer = ();

    fn mutable(
        value: &mut Self::Base,
        _buffer: &mut Self::Buffer,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        _id: egui::Id,
        _env: InspectorUi<'_, '_>,
    ) -> bool {
        ui.checkbox(value, "").changed()
    }

    fn readonly(
        value: &Self::Base,
        _buffer: &Self::Buffer,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        _id: egui::Id,
        _env: InspectorUi<'_, '_>,
    ) {
        let mut val = *value;
        ui.add_enabled_ui(false, |ui| ui.checkbox(&mut val, ""));
    }
}

impl MakeBuffer<()> for bool {
    fn make_buffer(&self) -> () {
        ()
    }
}
