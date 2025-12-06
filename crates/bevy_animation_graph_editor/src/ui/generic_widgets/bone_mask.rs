use bevy_animation_graph::core::edge_data::bone_mask::{BoneMask, BoneMaskType};

use crate::ui::generic_widgets::{
    bone_id::{BoneIdReadonlyWidget, BoneIdWidget},
    hashmap::HashMapWidget,
    picker::PickerWidget,
};

pub struct BoneMaskWidget<'a> {
    pub bone_mask: &'a mut BoneMask,
    pub id_hash: egui::Id,
}

impl<'a> BoneMaskWidget<'a> {
    pub fn new_salted(bone_mask: &'a mut BoneMask, salt: impl std::hash::Hash) -> Self {
        Self {
            bone_mask,
            id_hash: egui::Id::new(salt),
        }
    }
}

impl<'a> egui::Widget for BoneMaskWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let previous_base = self.bone_mask.base;
            let mut response = PickerWidget::new_salted("bone mask type")
                .ui(ui, format!("{:?}", self.bone_mask.base), |ui| {
                    for val in [BoneMaskType::Positive, BoneMaskType::Negative] {
                        ui.selectable_value(&mut self.bone_mask.base, val, format!("{:?}", val));
                    }
                })
                .response;
            if previous_base != self.bone_mask.base {
                response.mark_changed();
            }

            response |= HashMapWidget::new_salted(&mut self.bone_mask.weights, "bone mask weights")
                .ui(
                    ui,
                    |ui, key| ui.add(BoneIdWidget::new_salted(key, "bone mask bone id")),
                    |ui, key| ui.add(BoneIdReadonlyWidget::new_salted(key, "bone mask bone id")),
                    |ui, weight| ui.add(egui::DragValue::new(weight).speed(0.01)),
                );

            response
        })
        .inner
    }
}
