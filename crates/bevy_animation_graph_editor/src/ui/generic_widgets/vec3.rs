use bevy::math::Vec3;

pub struct Vec3Widget<'a> {
    pub vec3: &'a mut Vec3,
    pub slider_step_size: f32,
    pub id_hash: egui::Id,
    pub width: f32,
}

impl<'a> Vec3Widget<'a> {
    pub fn new_salted(vec3: &'a mut Vec3, salt: impl std::hash::Hash) -> Self {
        Self {
            vec3,
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

impl<'a> egui::Widget for Vec3Widget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let mut total_size = ui.available_size();
            total_size.x = self.width;
            ui.horizontal(|ui| {
                let x_id = ui.id().with(self.id_hash).with("vec3 x");
                let y_id = ui.id().with(self.id_hash).with("vec3 y");
                let z_id = ui.id().with(self.id_hash).with("vec3 z");

                let x_response = ui
                    .push_id(x_id, |ui| {
                        ui.add_sized(
                            egui::Vec2::new(total_size.x / 3.1, total_size.y),
                            egui::DragValue::new(&mut self.vec3.x).speed(self.slider_step_size),
                        )
                    })
                    .inner;
                let y_response = ui
                    .push_id(y_id, |ui| {
                        ui.add_sized(
                            egui::Vec2::new(total_size.x / 3.1, total_size.y),
                            egui::DragValue::new(&mut self.vec3.y).speed(self.slider_step_size),
                        )
                    })
                    .inner;
                let z_response = ui
                    .push_id(z_id, |ui| {
                        ui.add_sized(
                            egui::Vec2::new(total_size.x / 3.1, total_size.y),
                            egui::DragValue::new(&mut self.vec3.z).speed(self.slider_step_size),
                        )
                    })
                    .inner;
                x_response | y_response | z_response
            })
            .inner
        })
        .inner
    }
}
