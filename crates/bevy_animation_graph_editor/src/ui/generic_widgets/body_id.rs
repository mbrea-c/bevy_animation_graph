use bevy_animation_graph::core::ragdoll::definition::{BodyId, Ragdoll};
use uuid::Uuid;

pub struct BodyIdWidget<'a> {
    pub body_id: &'a mut BodyId,
    pub id_hash: egui::Id,
    pub ragdoll: Option<&'a Ragdoll>,
}

impl<'a> BodyIdWidget<'a> {
    pub fn new_salted(body_id: &'a mut BodyId, salt: impl std::hash::Hash) -> Self {
        Self {
            body_id,
            id_hash: egui::Id::new(salt),
            ragdoll: None,
        }
    }

    pub fn with_ragdoll(mut self, ragdoll: &'a Ragdoll) -> Self {
        self.ragdoll = Some(ragdoll);
        self
    }
}

impl<'a> egui::Widget for BodyIdWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let mut response = ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());
            if let Some(ragdoll) = self.ragdoll {
                let popup_response = ui
                    .menu_button("🔍", |ui| {
                        for body in ragdoll.iter_bodies() {
                            let label = if body.label.is_empty() {
                                format!("{}", body.id.uuid().hyphenated())
                            } else {
                                body.label.clone()
                            };
                            if ui.button(label).clicked() {
                                *self.body_id = body.id;
                                response.mark_changed();
                            }
                        }
                    })
                    .response;

                response |= popup_response;
            }

            let mut uuid_str = format!("{}", self.body_id.uuid().hyphenated());
            response |= ui.text_edit_singleline(&mut uuid_str);
            if let Ok(uuid) = Uuid::parse_str(&uuid_str) {
                *self.body_id = BodyId::from_uuid(uuid);
            }

            response
        })
        .inner
    }
}
