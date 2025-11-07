use std::f32::consts::PI;

use bevy_animation_graph::core::ragdoll::definition::AngleLimit;

pub enum AngleUnit {
    Deg,
    #[allow(dead_code)]
    Rad,
}

impl AngleUnit {
    pub fn factor_to(&self) -> f32 {
        match self {
            AngleUnit::Deg => 180. / PI,
            AngleUnit::Rad => 1.,
        }
    }

    pub fn factor_from(&self) -> f32 {
        match self {
            AngleUnit::Deg => PI / 180.,
            AngleUnit::Rad => 1.,
        }
    }
}

pub struct AngleLimitWidget<'a> {
    pub angle_limit: &'a mut AngleLimit,
    pub slider_step_size: f32,
    pub angle_unit: AngleUnit,
    pub id_hash: egui::Id,
    pub width: f32,
}

impl<'a> AngleLimitWidget<'a> {
    pub fn new_salted(angle_limit: &'a mut AngleLimit, salt: impl std::hash::Hash) -> Self {
        Self {
            angle_limit,
            angle_unit: AngleUnit::Deg,
            slider_step_size: 0.01,
            id_hash: egui::Id::new(salt),
            width: 300.,
        }
    }

    #[allow(dead_code)]
    pub fn with_step_size(mut self, step_size: f32) -> Self {
        self.slider_step_size = step_size;
        self
    }

    #[allow(dead_code)]
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    #[allow(dead_code)]
    pub fn with_angle_unit(mut self, unit: AngleUnit) -> Self {
        self.angle_unit = unit;
        self
    }
}

impl<'a> egui::Widget for AngleLimitWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let mut total_size = ui.available_size();
            total_size.x = self.width;

            let mut angle_min = self.angle_limit.min * self.angle_unit.factor_to();
            let mut angle_max = self.angle_limit.max * self.angle_unit.factor_to();

            ui.horizontal(|ui| {
                let mut response = ui
                    .push_id("angle limit min", |ui| {
                        ui.add_sized(
                            egui::Vec2::new(total_size.x / 3.1, total_size.y),
                            egui::DragValue::new(&mut angle_min).speed(self.slider_step_size),
                        )
                    })
                    .inner;
                response |= ui
                    .push_id("angle limit max", |ui| {
                        ui.add_sized(
                            egui::Vec2::new(total_size.x / 3.1, total_size.y),
                            egui::DragValue::new(&mut angle_max).speed(self.slider_step_size),
                        )
                    })
                    .inner;

                self.angle_limit.min = angle_min * self.angle_unit.factor_from();
                self.angle_limit.max = angle_max * self.angle_unit.factor_from();

                response
            })
            .inner
        })
        .inner
    }
}
