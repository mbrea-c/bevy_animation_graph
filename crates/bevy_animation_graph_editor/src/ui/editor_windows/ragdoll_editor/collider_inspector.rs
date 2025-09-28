use bevy::ecs::world::World;
use bevy_animation_graph::core::ragdoll::definition::Collider;
use egui::Widget;

use crate::ui::{
    core::EditorWindowContext,
    generic_widgets::{isometry3d::Isometry3dWidget, u32_flags::U32Flags},
    utils::using_inspector_env,
};

pub struct ColliderInspector<'a, 'b> {
    pub world: &'a mut World,
    pub ctx: &'a mut EditorWindowContext<'b>,
    pub collider: &'a mut Collider,
}

impl Widget for ColliderInspector<'_, '_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = egui::Grid::new("ragdoll body inspector").show(ui, |ui| {
            let mut response = ui.label("ID:");
            response |= ui.label(format!("{:?}", self.collider.id));
            ui.end_row();

            response |= ui.label("label:");
            response |= ui.text_edit_singleline(&mut self.collider.label);
            ui.end_row();

            response |= ui.add(
                Isometry3dWidget::new_salted(&mut self.collider.local_offset, "isometry")
                    .with_step_size(0.01)
                    .with_flatten_grid(true),
            );

            response |= ui.label("layer memberships:");
            response |= ui.add(U32Flags::new_salted(
                &mut self.collider.layer_membership,
                "layer memberships",
            ));
            ui.end_row();

            response |= ui.label("layer filters:");
            response |= ui.add(U32Flags::new_salted(
                &mut self.collider.layer_filter,
                "layer filters",
            ));
            ui.end_row();

            response |= ui.label("override layers:");
            response |= ui.add(egui::Checkbox::without_text(
                &mut self.collider.override_layers,
            ));
            ui.end_row();

            // TODO: Figure out a better way to handle enum UIs
            response |= ui.label("shape:");
            let shape_changed = using_inspector_env(self.world, |mut env| {
                env.ui_for_reflect(&mut self.collider.shape, ui)
            });
            if shape_changed {
                response.mark_changed();
            }

            response
        });

        response.inner
    }
}
