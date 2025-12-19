use bevy::{ecs::world::World, utils::default};
use bevy_animation_graph::core::state_machine::high_level::{TransitionData, TransitionKind};

use crate::ui::generic_widgets::{asset_picker::AssetPicker, option::CheapOptionWidget};

pub struct TransitionDataWidget<'a> {
    pub transition_data: &'a mut TransitionData,
    pub world: &'a mut World,
    pub id_hash: egui::Id,
    pub width: f32,
}

impl<'a> TransitionDataWidget<'a> {
    pub fn new_salted(
        transition_data: &'a mut TransitionData,
        world: &'a mut World,
        salt: impl std::hash::Hash,
    ) -> Self {
        Self {
            transition_data,
            world,
            id_hash: egui::Id::new(salt),
            width: 300.,
        }
    }

    #[allow(dead_code)]
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl<'a> egui::Widget for TransitionDataWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        egui::Grid::new(self.id_hash)
            .show(ui, |ui| {
                let mut response = ui.label("transition kind:");
                let mut tag = match &self.transition_data.kind {
                    TransitionKind::Immediate => TransitionKindTag::Immediate,
                    TransitionKind::Graph { .. } => TransitionKindTag::Graph,
                };
                let original = tag;
                response |= egui::ComboBox::new("transition kind", format!("{:?}", tag))
                    .show_ui(ui, |ui| {
                        let mut val = |t| ui.selectable_value(&mut tag, t, format!("{:?}", t));
                        val(TransitionKindTag::Immediate);
                        val(TransitionKindTag::Graph);
                    })
                    .response;

                if tag != original {
                    response.mark_changed();
                    match tag {
                        TransitionKindTag::Immediate => {
                            self.transition_data.kind = TransitionKind::Immediate;
                        }
                        TransitionKindTag::Graph => {
                            self.transition_data.kind = TransitionKind::Graph {
                                graph: default(),
                                timed: default(),
                            };
                        }
                    }
                }
                ui.end_row();

                match &mut self.transition_data.kind {
                    TransitionKind::Immediate => {}
                    TransitionKind::Graph { graph, timed } => {
                        response |= ui.label("transition graph:");
                        response |= ui.add(AssetPicker::new_salted(
                            graph,
                            self.world,
                            "state transition graph handle",
                        ));
                        ui.end_row();

                        response |= ui.label("timed:");
                        response |= ui
                            .horizontal(|ui| {
                                CheapOptionWidget::new_salted(timed, "timed widget")
                                    .ui(ui, |ui, val| ui.add(egui::DragValue::new(val)))
                            })
                            .inner;
                    }
                }

                response
            })
            .inner
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransitionKindTag {
    Immediate,
    Graph,
}
