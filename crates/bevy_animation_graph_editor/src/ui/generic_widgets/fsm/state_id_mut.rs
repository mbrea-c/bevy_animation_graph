use bevy_animation_graph::core::state_machine::high_level::{StateId, StateMachine};

use crate::ui::generic_widgets::uuid::UuidWidget;

pub struct StateIdWidget<'a> {
    pub state_id: &'a mut StateId,
    pub id_hash: egui::Id,
    pub fsm: Option<&'a StateMachine>,
}

impl<'a> StateIdWidget<'a> {
    pub fn new(state_id: &'a mut StateId) -> Self {
        Self {
            state_id,
            id_hash: egui::Id::new(0),
            fsm: None,
        }
    }

    pub fn salted(mut self, salt: impl std::hash::Hash) -> Self {
        self.id_hash = egui::Id::new(salt);
        self
    }

    pub fn with_fsm(mut self, fsm: Option<&'a StateMachine>) -> Self {
        self.fsm = fsm;
        self
    }
}

impl<'a> egui::Widget for StateIdWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let mut response = ui
                    .horizontal(|ui| {
                        let picker_response = self.fsm.map(|r| picker(ui, self.state_id, r));

                        let mut uuid = self.state_id.uuid();
                        let mut response =
                            ui.add(UuidWidget::new_salted(&mut uuid, "body id uuid"));
                        if let Some(picker_response) = picker_response {
                            response |= picker_response;
                        }
                        *self.state_id = StateId::from(uuid);

                        response
                    })
                    .inner;

                if let Some(fsm) = self.fsm
                    && let Some(state) = fsm.states.get(self.state_id)
                {
                    response |= ui.label(&state.label);
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

fn picker(ui: &mut egui::Ui, body_id: &mut StateId, fsm: &StateMachine) -> egui::Response {
    let mut changed = false;
    let mut popup_response = ui
        .menu_button("🔍", |ui| {
            for state in fsm.states.values() {
                let label = if state.label.is_empty() {
                    format!("{}", state.id.uuid().hyphenated())
                } else {
                    state.label.clone()
                };
                if ui.button(label).clicked() {
                    *body_id = state.id;
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

// pub struct BodyIdReadonlyWidget<'a> {
//     pub body_id: &'a BodyId,
//     pub id_hash: egui::Id,
//     pub ragdoll: Option<&'a Ragdoll>,
// }
//
// impl<'a> BodyIdReadonlyWidget<'a> {
//     pub fn new_salted(body_id: &'a BodyId, salt: impl std::hash::Hash) -> Self {
//         Self {
//             body_id,
//             id_hash: egui::Id::new(salt),
//             ragdoll: None,
//         }
//     }
//
//     pub fn with_ragdoll(mut self, ragdoll: Option<&'a Ragdoll>) -> Self {
//         self.ragdoll = ragdoll;
//         self
//     }
// }
//
// impl<'a> egui::Widget for BodyIdReadonlyWidget<'a> {
//     fn ui(self, ui: &mut egui::Ui) -> egui::Response {
//         ui.push_id(self.id_hash, |ui| {
//             ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
//                 let mut response = ui.label(format!("{}", self.body_id.uuid().hyphenated()));
//
//                 if let Some(ragdoll) = self.ragdoll
//                     && let Some(body) = ragdoll.get_body(*self.body_id)
//                 {
//                     response |= ui.label(&body.label);
//                 } else {
//                     response |= ui.label("No label available");
//                 }
//
//                 response
//             })
//             .inner
//         })
//         .inner
//     }
// }
