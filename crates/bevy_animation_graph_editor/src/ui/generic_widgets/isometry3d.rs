use bevy::math::{Isometry3d, Vec3};

use crate::ui::generic_widgets::{quat::QuatWidget, vec3::Vec3Widget};

pub struct Isometry3dWidget<'a> {
    pub isometry: &'a mut Isometry3d,
    pub slider_step_size: f32,
    pub id_hash: egui::Id,
    pub flatten_grid: bool,
}

impl<'a> Isometry3dWidget<'a> {
    pub fn new_salted(isometry: &'a mut Isometry3d, salt: impl std::hash::Hash) -> Self {
        Self {
            isometry,
            slider_step_size: 0.1,
            id_hash: egui::Id::new(salt),
            flatten_grid: false,
        }
    }

    pub fn with_step_size(mut self, step_size: f32) -> Self {
        self.slider_step_size = step_size;
        self
    }

    pub fn with_flatten_grid(mut self, flatten_grid: bool) -> Self {
        self.flatten_grid = flatten_grid;
        self
    }
}

impl<'a> egui::Widget for Isometry3dWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut draw = |ui: &mut egui::Ui| {
            let translation_label_response = ui.label("translation:");
            let mut translation: Vec3 = self.isometry.translation.into();
            let translation_response = ui.add(
                Vec3Widget::new_salted(&mut translation, self.id_hash.with("translation"))
                    .with_step_size(self.slider_step_size)
                    .with_width(200.),
            );
            self.isometry.translation = translation.into();
            ui.end_row();

            let rotation_label_response = ui.label("rotation:");
            let rotation_response = ui.add(
                QuatWidget::new_salted(&mut self.isometry.rotation, self.id_hash.with("rotation"))
                    .with_step_size(self.slider_step_size)
                    .with_width(200.),
            );
            ui.end_row();

            translation_label_response
                | translation_response
                | rotation_label_response
                | rotation_response
        };

        if self.flatten_grid {
            draw(ui)
        } else {
            egui::Grid::new(self.id_hash).show(ui, |ui| draw(ui)).inner
        }
    }
}
