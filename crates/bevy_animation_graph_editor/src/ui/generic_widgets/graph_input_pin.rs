use bevy_animation_graph::core::{
    animation_graph::{GraphInputPin, PinId},
    state_machine::low_level::FsmBuiltinPin,
};

use crate::ui::generic_widgets::picker::PickerWidget;

pub struct GraphInputPinWidget<'a> {
    pub graph_input_pin: &'a mut GraphInputPin,
    pub id_hash: egui::Id,
}

impl<'a> GraphInputPinWidget<'a> {
    pub fn new_salted(graph_input_pin: &'a mut GraphInputPin, salt: impl std::hash::Hash) -> Self {
        Self {
            graph_input_pin,
            id_hash: egui::Id::new(salt),
        }
    }
}

impl<'a> egui::Widget for GraphInputPinWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let mut selected = GraphInputPinType::from(&*self.graph_input_pin);
            let mut response = PickerWidget::new_salted("pin type picker")
                .ui(ui, format!("{:?}", selected), |ui| {
                    for val in [
                        GraphInputPinType::Passthrough,
                        GraphInputPinType::FsmSource,
                        GraphInputPinType::FsmTarget,
                        GraphInputPinType::FsmBuiltin,
                    ] {
                        ui.selectable_value(&mut selected, val, val.to_string());
                    }
                })
                .response;

            if selected != GraphInputPinType::from(&*self.graph_input_pin) {
                response.mark_changed();
            }

            if response.changed() {
                *self.graph_input_pin = selected.initialize();
            }

            match self.graph_input_pin {
                GraphInputPin::Passthrough(pin_id)
                | GraphInputPin::FromFsmSource(pin_id)
                | GraphInputPin::FromFsmTarget(pin_id) => {
                    response |= ui.add(egui::TextEdit::singleline(pin_id).desired_width(100.));
                }
                GraphInputPin::FsmBuiltin(fsm_builtin_pin) => {
                    let original = fsm_builtin_pin.clone();
                    response |= PickerWidget::new_salted("builtin pin picker")
                        .ui(ui, format!("{:?}", fsm_builtin_pin), |ui| {
                            let mut show = |val| {
                                let label = format!("{:?}", val);
                                ui.selectable_value(fsm_builtin_pin, val, label)
                            };
                            show(FsmBuiltinPin::PercentThroughDuration);
                            show(FsmBuiltinPin::TimeElapsed);
                        })
                        .response;

                    if &original != fsm_builtin_pin {
                        response.mark_changed();
                    }
                }
            }

            response
        })
        .inner
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GraphInputPinType {
    Passthrough,
    FsmSource,
    FsmTarget,
    FsmBuiltin,
}

impl From<&GraphInputPin> for GraphInputPinType {
    fn from(value: &GraphInputPin) -> Self {
        match value {
            GraphInputPin::Passthrough(_) => Self::Passthrough,
            GraphInputPin::FromFsmSource(_) => Self::FsmSource,
            GraphInputPin::FromFsmTarget(_) => Self::FsmTarget,
            GraphInputPin::FsmBuiltin(_) => Self::FsmBuiltin,
        }
    }
}

impl std::fmt::Display for GraphInputPinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl GraphInputPinType {
    pub fn initialize(&self) -> GraphInputPin {
        match self {
            GraphInputPinType::Passthrough => GraphInputPin::Passthrough(PinId::default()),
            GraphInputPinType::FsmSource => GraphInputPin::FromFsmSource(PinId::default()),
            GraphInputPinType::FsmTarget => GraphInputPin::FromFsmTarget(PinId::default()),
            GraphInputPinType::FsmBuiltin => GraphInputPin::FsmBuiltin(FsmBuiltinPin::default()),
        }
    }
}
