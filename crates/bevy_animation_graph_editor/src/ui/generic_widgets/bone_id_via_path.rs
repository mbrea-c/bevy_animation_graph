use bevy_animation_graph::core::{animation_clip::EntityPath, id::BoneId, skeleton::Skeleton};

use crate::ui::generic_widgets::entity_path::EntityPathWidget;

pub struct BoneIdWidget<'a> {
    pub bone_id: &'a mut BoneId,
    pub id_hash: egui::Id,
    pub skeleton: &'a Skeleton,
}

impl<'a> BoneIdWidget<'a> {
    pub fn new(bone_id: &'a mut BoneId, skeleton: &'a Skeleton) -> Self {
        Self {
            bone_id,
            id_hash: egui::Id::new("bone id widget"),
            skeleton,
        }
    }

    pub fn salted(mut self, salt: impl std::hash::Hash) -> Self {
        self.id_hash = egui::Id::new(salt);
        self
    }
}

impl<'a> egui::Widget for BoneIdWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let current_path = self.skeleton.id_to_path(*self.bone_id).unwrap_or_default();
            let mut buffer = Buffer::from_ui(ui, *self.bone_id, current_path);

            let response = ui.add(
                EntityPathWidget::new_salted(&mut buffer.path, "bone id entity path")
                    .with_options(self.skeleton.iter_paths()),
            );
            buffer.write_back(ui);

            if response.changed() {
                *self.bone_id = buffer.path.id();
            }

            response
        })
        .inner
    }
}

#[derive(Clone)]
pub struct Buffer {
    original_id: BoneId,
    path: EntityPath,
}

impl Buffer {
    pub fn from_ui(ui: &mut egui::Ui, bone_id: BoneId, path: EntityPath) -> Self {
        let id = Self::id(ui);
        ui.memory_mut(|mem| {
            let new = move || Self {
                original_id: bone_id,
                path: path.clone(),
            };
            let val = mem.data.get_temp_mut_or_insert_with(id, &new).clone();
            if val.original_id == bone_id {
                val
            } else {
                new()
            }
        })
    }
    pub fn write_back(&self, ui: &mut egui::Ui) {
        let id = Self::id(ui);
        ui.memory_mut(|mem| mem.data.insert_temp(id, self.clone()));
    }

    fn id(ui: &egui::Ui) -> egui::Id {
        ui.id().with("bone id buffer")
    }
}
