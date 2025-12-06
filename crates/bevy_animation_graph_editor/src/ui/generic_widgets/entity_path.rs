use bevy_animation_graph::core::animation_clip::EntityPath;

pub struct EntityPathWidget<'a> {
    pub entity_path: &'a mut EntityPath,
    pub id_hash: egui::Id,
}

impl<'a> EntityPathWidget<'a> {
    pub fn new_salted(entity_path: &'a mut EntityPath, salt: impl std::hash::Hash) -> Self {
        Self {
            entity_path,
            id_hash: egui::Id::new(salt),
        }
    }
}

impl<'a> egui::Widget for EntityPathWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let mut data = self.entity_path.to_slashed_string();

            let response = ui.text_edit_singleline(&mut data);
            if response.changed() {
                *self.entity_path = EntityPath::from_slashed_string(data);
            }

            response
        })
        .inner
    }
}
