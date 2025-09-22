use bevy::ecs::world::World;
use bevy_animation_graph::core::ragdoll::definition::{Body, BodyMode};
use egui::Widget;

use crate::ui::{core::EditorWindowContext, generic_widgets::isometry3d::Isometry3dWidget};

pub struct BodyInspector<'a, 'b> {
    pub world: &'a mut World,
    pub ctx: &'a mut EditorWindowContext<'b>,
    pub body: &'a mut Body,
}

impl Widget for BodyInspector<'_, '_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = egui::Grid::new("ragdoll body inspector").show(ui, |ui| {
            ui.label("ID");
            ui.label(format!("{:?}", self.body.id));
            ui.end_row();

            let isometry_response = ui.add(
                Isometry3dWidget::new_salted(&mut self.body.isometry, "isometry")
                    .with_step_size(0.01)
                    .with_flatten_grid(true),
            );

            ui.label("Body mode:");
            let body_mode_response = egui::ComboBox::from_id_salt("body mode")
                .selected_text(match self.body.default_mode {
                    BodyMode::Kinematic => "Kinematic",
                    BodyMode::Dynamic => "Dynamic",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.body.default_mode,
                        BodyMode::Kinematic,
                        "Kinematic",
                    );
                    ui.selectable_value(&mut self.body.default_mode, BodyMode::Dynamic, "Dynamic");
                })
                .response;

            isometry_response | body_mode_response
        });

        response.inner
    }
}
