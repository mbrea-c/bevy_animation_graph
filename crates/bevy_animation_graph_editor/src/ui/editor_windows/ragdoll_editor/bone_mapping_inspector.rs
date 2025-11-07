use bevy::ecs::world::World;
use bevy_animation_graph::core::ragdoll::{bone_mapping::BoneMapping, definition::Ragdoll};
use egui::Widget;

use crate::ui::{
    core::LegacyEditorWindowContext,
    generic_widgets::{body_id::BodyIdWidget, isometry3d::Isometry3dWidget, list::ListWidget},
};

pub struct BoneMappingInspector<'a, 'b> {
    pub world: &'a mut World,
    pub ctx: &'a mut LegacyEditorWindowContext<'b>,
    pub bone: &'a mut BoneMapping,
    pub ragdoll: &'a Ragdoll,
}

impl Widget for BoneMappingInspector<'_, '_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(format!("ID: {:?}", self.bone.bone_id.id()));
        ui.label(format!("Path: {}", self.bone.bone_id.to_slashed_string()));
        ListWidget::new_salted(&mut self.bone.bodies, "body weight list").ui(
            ui,
            |ui, body_weight| {
                egui::Grid::new("body weight grid")
                    .show(ui, |ui| {
                        let mut response = ui.label("body ID:");
                        response |= ui.add(
                            BodyIdWidget::new_salted(&mut body_weight.body, "body id picker")
                                .with_ragdoll(Some(self.ragdoll)),
                        );
                        ui.end_row();

                        response |= ui.label("weight:");
                        response |= ui.add(egui::DragValue::new(&mut body_weight.weight));
                        ui.end_row();

                        response |= ui
                            .label("override offset?:")
                            .on_hover_text(include_str!("../../../tooltips/override_offset.txt"));
                        response |= ui.add(egui::Checkbox::without_text(
                            &mut body_weight.override_offset,
                        ));
                        ui.end_row();

                        if body_weight.override_offset {
                            response |= ui.add(
                                Isometry3dWidget::new_salted(
                                    &mut body_weight.offset,
                                    "offset from body",
                                )
                                .with_flatten_grid(true),
                            );

                            ui.end_row();
                        }

                        response
                    })
                    .inner
            },
        )
    }
}
