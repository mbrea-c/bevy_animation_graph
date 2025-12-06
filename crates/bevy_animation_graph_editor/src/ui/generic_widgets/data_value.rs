use bevy_animation_graph::core::edge_data::{DataSpec, DataValue};

use crate::ui::generic_widgets::{
    bone_mask::BoneMaskWidget, entity_path::EntityPathWidget, picker::PickerWidget,
    quat::QuatWidget, ragdoll_config::RagdollConfigWidget, vec2::Vec2Widget, vec3::Vec3Widget,
};

pub struct DataValueWidget<'a> {
    pub data_value: &'a mut DataValue,
    pub id_hash: egui::Id,
}

impl<'a> DataValueWidget<'a> {
    pub fn new_salted(data_value: &'a mut DataValue, salt: impl std::hash::Hash) -> Self {
        Self {
            data_value,
            id_hash: egui::Id::new(salt),
        }
    }
}

impl<'a> egui::Widget for DataValueWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let mut selected = DataSpec::from(&*self.data_value);
            let mut response = PickerWidget::new_salted("data spec picker")
                .ui(ui, format!("{:?}", selected), |ui| {
                    for val in [
                        DataSpec::F32,
                        DataSpec::Bool,
                        DataSpec::Vec2,
                        DataSpec::Vec3,
                        DataSpec::EntityPath,
                        DataSpec::Quat,
                        DataSpec::BoneMask,
                        DataSpec::Pose,
                        DataSpec::EventQueue,
                        DataSpec::RagdollConfig,
                    ] {
                        ui.selectable_value(&mut selected, val, format!("{:?}", val));
                    }
                })
                .response;

            if selected != DataSpec::from(&*self.data_value) {
                response.mark_changed();
                *self.data_value = DataValue::default_from_spec(selected);
            }

            match self.data_value {
                DataValue::F32(val) => {
                    response |= ui.add(egui::DragValue::new(val));
                }
                DataValue::Bool(val) => {
                    response |= ui.add(egui::Checkbox::without_text(val));
                }
                DataValue::Vec2(vec2) => {
                    response |= ui.add(Vec2Widget::new_salted(vec2, "vec2"));
                }
                DataValue::Vec3(vec3) => {
                    response |= ui.add(Vec3Widget::new_salted(vec3, "vec3"));
                }
                DataValue::Quat(quat) => {
                    response |= ui.add(QuatWidget::new_salted(quat, "quat"));
                }
                DataValue::EntityPath(entity_path) => {
                    response |= ui.add(EntityPathWidget::new_salted(entity_path, "entity path"));
                }
                DataValue::BoneMask(bone_mask) => {
                    response |= ui.add(BoneMaskWidget::new_salted(bone_mask, "bone mask"));
                }
                DataValue::Pose(_) => {
                    response |= ui.label("Pose value editing not supported");
                }
                DataValue::EventQueue(_) => {
                    response |= ui.label("Event queue editing not supported");
                }
                DataValue::RagdollConfig(ragdoll_config) => {
                    response |= ui.add(RagdollConfigWidget::new_salted(
                        ragdoll_config,
                        "ragdoll config",
                    ));
                }
            }

            response
        })
        .inner
    }
}
