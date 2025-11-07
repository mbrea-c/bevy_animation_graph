use bevy_animation_graph::core::ragdoll::definition::{Body, BodyMode};
use egui::Widget;

use crate::ui::generic_widgets::vec3::Vec3Widget;

pub struct BodyInspector<'a> {
    pub body: &'a mut Body,
}

impl Widget for BodyInspector<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = egui::Grid::new("ragdoll body inspector").show(ui, |ui| {
            let mut response = ui.label("ID");
            response |= ui.label(format!("{:?}", self.body.id));
            ui.end_row();

            response |= ui.label("label:");
            response |= ui.text_edit_singleline(&mut self.body.label);
            ui.end_row();

            response |= ui.label("offset:");
            response |= ui
                .add(Vec3Widget::new_salted(&mut self.body.offset, "offset").with_step_size(0.005));
            ui.end_row();

            response |= ui.label("Body mode:");
            response |= egui::ComboBox::from_id_salt("body mode")
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
            ui.end_row();

            response |= ui.label("use symmetry:");
            response |= ui.add(egui::Checkbox::without_text(&mut self.body.use_symmetry));

            response
        });

        response.inner
    }
}
