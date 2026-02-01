use bevy::ecs::world::World;
use bevy_animation_graph::core::state_machine::high_level::{DirectTransition, StateMachine};

use crate::ui::generic_widgets::fsm::{
    state_id_mut::StateIdWidget, transition_data::TransitionDataWidget,
};

pub struct DirectTransitionWidget<'a> {
    pub direct_transition: &'a mut DirectTransition,
    pub world: &'a mut World,
    pub id_hash: egui::Id,
    pub fsm: Option<&'a StateMachine>,
}

impl<'a> DirectTransitionWidget<'a> {
    pub fn new(direct_transition: &'a mut DirectTransition, world: &'a mut World) -> Self {
        Self {
            direct_transition,
            world,
            id_hash: egui::Id::NULL,
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

impl<'a> egui::Widget for DirectTransitionWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let mut response = egui::Grid::new(self.id_hash)
                .show(ui, |ui| {
                    let mut response = ui.label("id:");
                    response |=
                        ui.label(format!("{}", self.direct_transition.id.uuid().hyphenated()));
                    ui.end_row();

                    response |= ui.label("source:");
                    response |= ui.add(
                        StateIdWidget::new(&mut self.direct_transition.source)
                            .salted("source")
                            .with_fsm(self.fsm),
                    );
                    ui.end_row();

                    response |= ui.label("target:");
                    response |= ui.add(
                        StateIdWidget::new(&mut self.direct_transition.target)
                            .salted("target")
                            .with_fsm(self.fsm),
                    );
                    ui.end_row();

                    response
                })
                .inner;

            response |= egui::Frame::new()
                .stroke(egui::Stroke {
                    width: 1.,
                    color: ui.visuals().weak_text_color(),
                })
                .fill(ui.visuals().faint_bg_color)
                .outer_margin(3.)
                .inner_margin(4.)
                .corner_radius(5.)
                .show(ui, |ui| {
                    ui.add(TransitionDataWidget::new_salted(
                        &mut self.direct_transition.data,
                        self.world,
                        "transition data",
                    ))
                })
                .inner;
            response
        })
        .inner
    }
}
