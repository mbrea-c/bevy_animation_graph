use bevy::math::{EulerRot, Quat, Vec3};

use crate::ui::generic_widgets::vec3::Vec3Widget;

pub struct QuatWidget<'a> {
    pub quat: &'a mut Quat,
    pub slider_step_size: f32,
    pub id_hash: egui::Id,
    pub width: f32,
}

impl<'a> QuatWidget<'a> {
    pub fn new_salted(quat: &'a mut Quat, salt: impl std::hash::Hash) -> Self {
        Self {
            quat,
            slider_step_size: 0.1,
            id_hash: egui::Id::new(salt),
            width: 300.,
        }
    }

    pub fn with_step_size(mut self, step_size: f32) -> Self {
        self.slider_step_size = step_size;
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl<'a> egui::Widget for QuatWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            let mut euler_vec: Vec3 = self.quat.to_euler(EulerRot::XYZ).into();

            let response = ui.add(Vec3Widget {
                vec3: &mut euler_vec,
                slider_step_size: self.slider_step_size,
                id_hash: self.id_hash.with("quat as vec"),
                width: self.width,
            });

            *self.quat = Quat::from_euler(EulerRot::XYZ, euler_vec.x, euler_vec.y, euler_vec.z);

            response
        })
        .inner
    }
}
