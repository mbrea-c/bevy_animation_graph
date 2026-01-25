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
            let buffer_id = ui.id().with("entity path slashed string buffer");

            // clear buffer if outdated
            ui.memory_mut(|mem| {
                let prev = mem.data.get_temp::<Buffer>(buffer_id);
                if let Some(prev) = prev
                    && &prev.original != self.entity_path
                {
                    mem.data.remove_temp::<Buffer>(buffer_id);
                }
            });

            let mut buffer = ui.memory_mut(|mem| {
                mem.data
                    .get_temp_mut_or_insert_with(buffer_id, || Buffer {
                        value: self.entity_path.to_slashed_string(),
                        original: self.entity_path.clone(),
                    })
                    .clone()
            });

            let response = ui.text_edit_singleline(&mut buffer.value);

            if response.changed() {
                *self.entity_path = EntityPath::from_slashed_string(buffer.value.clone());
            }

            ui.memory_mut(|mem| mem.data.insert_temp(buffer_id, buffer));

            response
        })
        .inner
    }
}

#[derive(Clone, Default)]
struct Buffer {
    value: String,
    original: EntityPath,
}
