use bevy::reflect::PartialReflect;
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui_dock::egui;

pub struct WrapUi<'a, 'c> {
    inspector_ui: InspectorUi<'a, 'c>,
}

impl WrapUi<'_, '_> {
    pub fn mutable_buffered<T: PartialReflect, O>(
        &mut self,
        value: &T,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &O,
    ) -> Option<T> {
        let is_changed = self
            .inspector_ui
            .ui_for_reflect_with_options(value, ui, id, options);
        is_changed.then(|| {
            // Need to get the buffered value, parse it into the real value, and return
        })
    }
}
