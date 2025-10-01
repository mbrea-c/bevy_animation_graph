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
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let mut response = ui
                    .horizontal(|ui| {
                        let picker_response = self.ragdoll.map(|r| picker(ui, self.body_id, r));

                        let mut uuid_str = format!("{}", self.body_id.uuid().hyphenated());
                        let mut response = ui.text_edit_singleline(&mut uuid_str);
                        if let Some(picker_response) = picker_response {
                            response |= picker_response;
                        }
                        if let Ok(uuid) = Uuid::parse_str(&uuid_str) {
                            *self.body_id = BodyId::from_uuid(uuid);
                        }

                        response
                    })
                    .inner;

                if let Some(ragdoll) = self.ragdoll
                    && let Some(body) = ragdoll.get_body(*self.body_id)
                {
                    response |= ui.label(&body.label);
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

fn picker(ui: &mut egui::Ui, body_id: &mut BodyId, ragdoll: &Ragdoll) -> egui::Response {
    let mut changed = false;
    let mut popup_response = ui
        .menu_button("🔍", |ui| {
            for body in ragdoll.iter_bodies() {
                let label = if body.label.is_empty() {
                    format!("{}", body.id.uuid().hyphenated())
                } else {
                    body.label.clone()
                };
                if ui.button(label).clicked() {
                    *body_id = body.id;
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
