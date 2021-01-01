use bevy::ecs::world::World;
use bevy_animation_graph::core::state_machine::high_level::State;

use crate::ui::generic_widgets::{
    asset_picker::AssetPicker, fsm::transition_data::TransitionDataWidget,
    option::CheapOptionWidget,
};

pub struct StateWidget<'a> {
    pub state: &'a mut State,
    pub world: &'a mut World,
    pub id_hash: egui::Id,
}

impl<'a> StateWidget<'a> {
    pub fn new_salted(
        state: &'a mut State,
        world: &'a mut World,
        salt: impl std::hash::Hash,
    ) -> Self {
        Self {
            state,
            world,
            id_hash: egui::Id::new(salt),
        }
    }
}

impl<'a> egui::Widget for StateWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = egui::Grid::new(self.id_hash)
            .show(ui, |ui| {
                let mut response = ui.label("id:");
                response |= ui.label(format!("{}", self.state.id.uuid().hyphenated()));
                ui.end_row();

                response |= ui.label("label:");
                response |= ui.text_edit_singleline(&mut self.state.label);
                ui.end_row();

                response |= ui.label("graph:");
                let path = self
                    .state
                    .graph
                    .path()
                    .map_or("<no asset path>".into(), |ap| ap.to_string());
                let r = ui.menu_button(path, |ui| {
                    ui.add(AssetPicker::new_salted(
                        &mut self.state.graph,
                        self.world,
                        ui.id().with("fsm state graph"),
                    ))
                });
                response |= r.response;
                if r.inner.is_some_and(|r| r.changed()) {
                    response.mark_changed();
                }
                ui.end_row();

                response
            })
            .inner;

        response |=
            CheapOptionWidget::new_salted(&mut self.state.state_transition, "state transition")
                .with_checkbox_label(Some("enable state transition:".into()))
                .ui(ui, |ui, data| {
                    egui::Frame::new()
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
                                data,
                                self.world,
                                "transition data",
                            ))
                        })
                        .inner
                });

        response
    }
}
