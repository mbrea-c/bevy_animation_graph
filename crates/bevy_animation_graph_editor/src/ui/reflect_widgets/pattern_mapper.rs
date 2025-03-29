use std::any::Any;

use bevy_animation_graph::prelude::config::{PatternMapper, PatternMapperSerial};
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui_dock::egui;

use super::EguiInspectorExtension;

#[derive(Default)]
pub struct PatternMapperInspector;

impl EguiInspectorExtension for PatternMapperInspector {
    type Base = PatternMapper;
    type Buffer = PatternMapperSerial;

    fn mutable(
        value: &mut Self::Base,
        buffer: Option<&mut Self::Buffer>,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) -> bool {
        let buffer = buffer.unwrap();

        match env.ui_for_reflect_with_options(buffer, ui, id, &()) {
            true => {
                if let Ok(mapper) = PatternMapper::try_from(buffer.clone()) {
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
        buffer: Option<&Self::Buffer>,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) {
        let buffer = buffer.unwrap();

        env.ui_for_reflect_readonly_with_options(buffer, ui, id, &());
    }

    fn init_buffer(value: &Self::Base) -> Option<Self::Buffer> {
        Some(value.clone().into())
    }

    fn needs_buffer() -> bool {
        true
    }
}
