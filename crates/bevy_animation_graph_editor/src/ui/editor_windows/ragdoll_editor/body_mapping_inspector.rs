use bevy_animation_graph::core::{ragdoll::bone_mapping::BodyMapping, skeleton::Skeleton};
use egui::Widget;

use crate::ui::generic_widgets::{bone_id::BoneIdWidget, isometry3d::Isometry3dWidget};

pub struct BodyMappingInspector<'a> {
    pub body_mapping: &'a mut BodyMapping,
    pub skeleton: &'a Skeleton,
}

impl Widget for BodyMappingInspector<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let BodyMapping { body_id, bone, .. } = self.body_mapping;

        egui::Grid::new("body weight grid")
            .show(ui, |ui| {
                let mut response = ui.label("ID:");
                response |= ui.label(format!("{}", body_id.uuid().hyphenated()));
                ui.end_row();

                response |= ui.label("target bone:");
                let mut bone_id = bone.bone.id();
                response |= ui.add(
                    BoneIdWidget::new_salted(&mut bone_id, "bone id picker")
                        .with_skeleton(Some(self.skeleton)),
                );
                if let Some(path) = self.skeleton.id_to_path(bone_id) {
                    bone.bone = path;
                }

                ui.end_row();

                response |= ui
                    .label("override offset?:")
                    .on_hover_text(include_str!("../../../tooltips/override_offset.txt"));
                response |= ui.add(egui::Checkbox::without_text(&mut bone.override_offset));
                ui.end_row();

                if bone.override_offset {
                    response |= ui.add(
                        Isometry3dWidget::new_salted(&mut bone.offset, "offset from bone")
                            .with_flatten_grid(true),
                    );

                    ui.end_row();
                }

                response
            })
            .inner
    }
}
