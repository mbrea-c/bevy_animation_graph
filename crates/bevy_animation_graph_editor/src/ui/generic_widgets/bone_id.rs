use bevy_animation_graph::core::{id::BoneId, skeleton::Skeleton};

use crate::ui::generic_widgets::uuid::UuidWidget;

pub struct BoneIdWidget<'a> {
    pub bone_id: &'a mut BoneId,
    pub id_hash: egui::Id,
    pub skeleton: Option<&'a Skeleton>,
}

impl<'a> BoneIdWidget<'a> {
    pub fn new_salted(bone_id: &'a mut BoneId, salt: impl std::hash::Hash) -> Self {
        Self {
            bone_id,
            id_hash: egui::Id::new(salt),
            skeleton: None,
        }
    }

    pub fn with_skeleton(mut self, skeleton: Option<&'a Skeleton>) -> Self {
        self.skeleton = skeleton;
        self
    }
}

impl<'a> egui::Widget for BoneIdWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let mut response = ui
                    .horizontal(|ui| {
                        let picker_response =
                            self.skeleton.map(|skn| picker(ui, self.bone_id, skn));

                        let mut uuid = self.bone_id.id();
                        let mut response =
                            ui.add(UuidWidget::new_salted(&mut uuid, "bone id uuid"));
                        if let Some(picker_response) = picker_response {
                            response |= picker_response;
                        }
                        *self.bone_id = BoneId::from_uuid(uuid);

                        response
                    })
                    .inner;

                if let Some(skeleton) = self.skeleton
                    && let Some(bone_path) = skeleton.id_to_path(*self.bone_id)
                {
                    response |= ui.label(bone_path.to_slashed_string());
                } else {
                    response |= ui.label("No label available");
                }

                response
            })
            .inner
        })
        .inner
    }
}

fn picker(ui: &mut egui::Ui, bone_id: &mut BoneId, skeleton: &Skeleton) -> egui::Response {
    let mut changed = false;
    let mut popup_response = ui
        .menu_button("🔍", |ui| {
            for available_bone_id in skeleton.iter_bones() {
                let Some(available_bone_path) = skeleton.id_to_path(available_bone_id) else {
                    continue;
                };
                let label = available_bone_path.to_slashed_string();
                if ui.button(label).clicked() {
                    *bone_id = available_bone_id;
                    changed = true;
                }
            }
        })
        .response;

    if changed {
        popup_response.mark_changed();
    }

    popup_response
}

pub struct BoneIdReadonlyWidget<'a> {
    pub bone_id: &'a BoneId,
    pub id_hash: egui::Id,
    pub skeleton: Option<&'a Skeleton>,
}

impl<'a> BoneIdReadonlyWidget<'a> {
    pub fn new_salted(bone_id: &'a BoneId, salt: impl std::hash::Hash) -> Self {
        Self {
            bone_id,
            id_hash: egui::Id::new(salt),
            skeleton: None,
        }
    }

    pub fn with_skeleton(mut self, skeleton: Option<&'a Skeleton>) -> Self {
        self.skeleton = skeleton;
        self
    }
}

impl<'a> egui::Widget for BoneIdReadonlyWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let mut response = ui.label(format!("{}", self.bone_id.id().hyphenated()));

                if let Some(skeleton) = self.skeleton
                    && let Some(bone_path) = skeleton.id_to_path(*self.bone_id)
                {
                    response |= ui.label(bone_path.to_slashed_string());
                } else {
                    response |= ui.label("No label available");
                }

                response
            })
            .inner
        })
        .inner
    }
}
