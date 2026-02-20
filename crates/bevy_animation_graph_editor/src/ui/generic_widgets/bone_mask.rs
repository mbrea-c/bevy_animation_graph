use bevy::platform::collections::HashMap;
use bevy_animation_graph::core::{
    animation_clip::EntityPath,
    edge_data::bone_mask::{BoneMask, BoneMaskType},
    id::BoneId,
    skeleton::Skeleton,
};

use crate::ui::generic_widgets::{
    entity_path::EntityPathWidget,
    hash_like::{HashLikeEditable, HashLikeWidget},
    picker::PickerWidget,
};

pub struct BoneMaskWidget<'a> {
    pub bone_mask: &'a mut BoneMask,
    pub id_hash: egui::Id,
    pub skeleton: Option<&'a Skeleton>,
}

impl<'a> BoneMaskWidget<'a> {
    pub fn new(bone_mask: &'a mut BoneMask) -> Self {
        Self {
            bone_mask,
            id_hash: egui::Id::new("bone mask widget"),
            skeleton: None,
        }
    }

    #[allow(dead_code)]
    pub fn salted(mut self, salt: impl std::hash::Hash) -> Self {
        self.id_hash = egui::Id::new(salt);
        self
    }

    pub fn with_skeleton(mut self, skeleton: Option<&'a Skeleton>) -> Self {
        self.skeleton = skeleton;
        self
    }
}

impl<'a> egui::Widget for BoneMaskWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let previous_base = self.bone_mask.base();
            let mut base = previous_base;
            let mut response = PickerWidget::new_salted("bone mask type")
                .ui(ui, format!("{:?}", base), |ui| {
                    for val in [BoneMaskType::Positive, BoneMaskType::Negative] {
                        ui.selectable_value(&mut base, val, format!("{:?}", val));
                    }
                })
                .response;
            if previous_base != base {
                self.bone_mask.set_base(base);
                response.mark_changed();
            }

            response |= HashLikeWidget::new_salted(
                &mut BoneMaskEditable {
                    bone_mask: self.bone_mask,
                    skeleton: self.skeleton,
                },
                "bone mask editable",
            )
            .ui(ui);

            response
        })
        .inner
    }
}

fn map_id_to_path(
    bone_id: BoneId,
    skeleton: Option<&Skeleton>,
    additional_bones: &HashMap<BoneId, EntityPath>,
) -> Option<EntityPath> {
    skeleton
        .and_then(|skn| skn.id_to_path(bone_id))
        .or_else(|| additional_bones.get(&bone_id).cloned())
}

fn with_bone_buffer(
    ui: &mut egui::Ui,
    bone_id: BoneId,
    skeleton: Option<&Skeleton>,
    additional_bones: &HashMap<BoneId, EntityPath>,
    f: impl FnOnce(&mut egui::Ui, &mut BoneBuffer) -> egui::Response,
    mut bone_changed: impl FnMut(BoneId, BoneId, EntityPath),
) -> egui::Response {
    let buffer_id = ui.id().with("bone id entity path wrapper");

    if let Some(mut buffer) = ui.memory_mut(|mem| {
        // Cleanup if mismatched
        if let Some(old_buffer) = mem.data.get_temp::<BoneBuffer>(buffer_id) {
            if old_buffer.original == bone_id {
                return Some(old_buffer);
            }
        }

        if let Some(entity_path) = map_id_to_path(bone_id, skeleton, additional_bones) {
            return Some(BoneBuffer {
                entity_path,
                original: bone_id,
            });
        }

        None
    }) {
        let response = f(ui, &mut buffer);

        if response.changed() {
            bone_changed(
                buffer.original,
                buffer.entity_path.id(),
                buffer.entity_path.clone(),
            );
        }

        ui.memory_mut(|mem| mem.data.insert_temp(buffer_id, buffer));

        response
    } else {
        ui.label(format!(
            "Could not map bone id {} to entity path",
            bone_id.id().hyphenated()
        ))
    }
}

#[derive(Clone, Default)]
pub struct BoneBuffer {
    entity_path: EntityPath,
    original: BoneId,
}

pub struct BoneMaskEditable<'a> {
    bone_mask: &'a mut BoneMask,
    skeleton: Option<&'a Skeleton>,
}

impl<'a> HashLikeEditable<BoneId, f32, EntityPath> for BoneMaskEditable<'a> {
    fn get(&self, key: &BoneId) -> Option<&f32> {
        self.bone_mask.get_weights().get(key)
    }

    fn keys<'b>(&'b self) -> impl Iterator<Item = &'b BoneId>
    where
        BoneId: 'b,
    {
        self.bone_mask.get_weights().keys()
    }

    fn add_new_value(&mut self, key: BoneId, value: f32, context: EntityPath) {
        self.bone_mask.add_bone_weight(key, context, value);
    }

    fn delete_existing_key(&mut self, key: &BoneId) {
        self.bone_mask.remove_bone_weight(*key);
    }

    fn edit_new_key(
        &mut self,
        ui: &mut egui::Ui,
        key: &mut BoneId,
        context: &mut EntityPath,
    ) -> egui::Response {
        if context.id() != *key {
            *key = context.id();
        }

        ui.add(
            EntityPathWidget::new_salted(context, "new key entity path bone mask")
                .with_options(self.skeleton.iter().flat_map(|skn| skn.iter_paths())),
        )
    }

    fn edit_new_value(
        &mut self,
        ui: &mut egui::Ui,
        value: &mut f32,
        _: &mut EntityPath,
    ) -> egui::Response {
        ui.add(egui::DragValue::new(value).speed(0.01))
    }

    fn edit_existing_value_for(&mut self, ui: &mut egui::Ui, key: &BoneId) -> egui::Response {
        let mut value = self.bone_mask.bone_weight(key);
        let response = ui.add(egui::DragValue::new(&mut value).speed(0.01));

        if response.changed() {
            self.bone_mask.update_bone_weight(*key, value);
        }

        response
    }

    fn edit_existing_key(&mut self, ui: &mut egui::Ui, key: &mut BoneId) -> egui::Response {
        let mut update = None;
        let mut response = with_bone_buffer(
            ui,
            *key,
            self.skeleton,
            self.bone_mask.get_paths(),
            |ui, buffer| {
                ui.add(
                    EntityPathWidget::new_salted(
                        &mut buffer.entity_path,
                        "bone mask entity path subwidget",
                    )
                    .with_options(self.skeleton.iter().flat_map(|skn| skn.iter_paths())),
                )
            },
            |old, new, path| {
                update = Some((old, new, path));
            },
        );

        if let Some((old, new, path)) = update
            && old != new
            && !self.bone_mask.contains_bone(new)
        {
            response.mark_changed();

            *key = new;

            let weight = self.bone_mask.bone_weight(&old);
            self.bone_mask.remove_bone_weight(old);
            self.bone_mask.add_bone_weight(new, path, weight);
        }

        response
    }
}
