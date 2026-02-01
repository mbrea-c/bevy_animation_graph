use std::any::Any;

use bevy_animation_graph::core::symmetry::{config::PatternMapper, serial::PatternMapperSerial};
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui_dock::egui;

use super::{EguiInspectorExtension, MakeBuffer};

#[derive(Default)]
pub struct PatternMapperInspector;

impl EguiInspectorExtension for PatternMapperInspector {
    type Base = PatternMapper;
    type Buffer = PatternMapperSerial;

    fn mutable(
        value: &mut Self::Base,
        buffer: &mut Self::Buffer,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) -> bool {
        match env.ui_for_reflect_with_options(buffer, ui, id, &()) {
            true => {
                if let Ok(mapper) = buffer.to_value() {
                    *value = mapper;
                    true
                } else {
                    false
                }
            }
            false => false,
        }
    }

    fn readonly(
        _value: &Self::Base,
        buffer: &Self::Buffer,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) {
        env.ui_for_reflect_readonly_with_options(buffer, ui, id, &());
    }
}

impl MakeBuffer<PatternMapperSerial> for PatternMapper {
    fn make_buffer(&self) -> PatternMapperSerial {
        PatternMapperSerial::from_value(self)
    }
}
