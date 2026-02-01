use bevy::math::Vec2;

pub struct Vec2Widget<'a> {
    pub vec2: &'a mut Vec2,
    pub slider_step_size: f32,
    pub id_hash: egui::Id,
    pub width: f32,
}

impl<'a> Vec2Widget<'a> {
    pub fn new_salted(vec2: &'a mut Vec2, salt: impl std::hash::Hash) -> Self {
        Self {
            vec2,
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
}

impl<'a> egui::Widget for Vec2Widget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let mut total_size = ui.available_size();
            total_size.x = self.width;
            ui.horizontal(|ui| {
                let x_id = ui.id().with(self.id_hash).with("vec3 x");
                let y_id = ui.id().with(self.id_hash).with("vec3 y");

                let x_response = ui
                    .push_id(x_id, |ui| {
                        ui.add_sized(
                            egui::Vec2::new(total_size.x / 3.1, total_size.y),
                            egui::DragValue::new(&mut self.vec2.x).speed(self.slider_step_size),
                        )
                    })
                    .inner;
                let y_response = ui
                    .push_id(y_id, |ui| {
                        ui.add_sized(
                            egui::Vec2::new(total_size.x / 3.1, total_size.y),
                            egui::DragValue::new(&mut self.vec2.y).speed(self.slider_step_size),
                        )
                    })
                    .inner;
                x_response | y_response
            })
            .inner
        })
        .inner
    }
}
