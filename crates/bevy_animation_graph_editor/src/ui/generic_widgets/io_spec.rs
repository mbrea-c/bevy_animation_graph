use bevy_animation_graph::core::{
    animation_graph::PinId,
    context::spec_context::{IoSpec, NodeInput, NodeInputPin, NodeOutput, NodeOutputPin},
    edge_data::DataSpec,
};

use crate::ui::generic_widgets::{data_spec_widget::DataSpecWidget, picker::PickerWidget};

pub struct IoSpecWidget<'a, I> {
    pub io_spec: &'a mut IoSpec<I>,
    pub id_hash: egui::Id,
}

impl<'a, I> IoSpecWidget<'a, I> {
    pub fn new_salted(io_spec: &'a mut IoSpec<I>, salt: impl std::hash::Hash) -> Self {
        Self {
            io_spec,
            id_hash: egui::Id::new(salt),
        }
    }
}

impl<'a, I: Clone + std::fmt::Debug + Eq + std::hash::Hash + Default + Send + Sync + 'static>
    IoSpecWidget<'a, I>
{
    pub fn show(
        mut self,
        ui: &mut egui::Ui,
        show_i: impl Fn(&mut egui::Ui, &mut I) -> egui::Response,
    ) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let mut response = egui::Frame::new()
                .outer_margin(3.)
                .inner_margin(3.)
                .corner_radius(3.)
                .stroke((1., ui.style().visuals.weak_text_color()))
                .show(ui, |ui| {
                    let mut response = ui.heading("Inputs");

                    for (i, input) in self.io_spec.sorted_inputs().into_iter().enumerate() {
                        ui.push_id(i, |ui| {
                            response |= self.show_input(ui, &show_i, input, i);
                        });
                    }

                    ui.horizontal(|ui| {
                        if ui.button("+").clicked() {
                            self.io_spec
                                .add_input_data(I::default(), DataSpec::default());
                        }
                        ui.label("Add item");
                    });

                    response
                })
                .inner;
            response |= egui::Frame::new()
                .outer_margin(3.)
                .inner_margin(3.)
                .corner_radius(3.)
                .stroke((1., ui.style().visuals.weak_text_color()))
                .show(ui, |ui| {
                    let mut response = ui.heading("Outputs");

                    for (i, output) in self.io_spec.sorted_outputs().into_iter().enumerate() {
                        ui.push_id(i, |ui| {
                            response |= self.show_output(ui, output, i);
                        });
                    }

                    ui.horizontal(|ui| {
                        if ui.button("+").clicked() {
                            self.io_spec
                                .add_output_data(PinId::default(), DataSpec::default());
                        }
                        ui.label("Add item");
                    });

                    response
                })
                .inner;

            response
        })
        .inner
    }

    fn show_input(
        &mut self,
        ui: &mut egui::Ui,
        show_i: impl Fn(&mut egui::Ui, &mut I) -> egui::Response,
        input: NodeInput<I>,
        index: usize,
    ) -> egui::Response {
        let input_key = NodeInputPin::from(input.clone());

        let buffer_id = ui.id().with("input buffer").with(&input_key);
        let mut buffer = ui.memory_mut(|mem| {
            mem.data
                .get_temp_mut_or_insert_with(buffer_id, || input.clone())
                .clone()
        });

        ui.horizontal(|ui| {
            let mut response = self.item_controls(
                ui,
                index,
                self.io_spec.len_input(),
                |this| {
                    this.io_spec.shift_input_index(&input_key, -1);
                },
                |this| {
                    this.io_spec.shift_input_index(&input_key, 1);
                },
                |this| {
                    this.io_spec.remove_input(&input_key);
                },
            );

            response |= self.show_input_time_data_selector(ui, &mut buffer);
            response |= match &mut buffer {
                NodeInput::Time(input) => self.show_input_time(ui, show_i, input),
                NodeInput::Data(input, data_spec) => {
                    self.show_input_data(ui, show_i, input, data_spec)
                }
            };

            ui.memory_mut(|mem| mem.data.insert_temp(buffer_id, buffer.clone()));

            if response.changed() && self.io_spec.update_input(&input_key, buffer.clone()) {
                ui.memory_mut(|mem| mem.data.remove_temp::<NodeInput<I>>(buffer_id));
            }

            response
        })
        .inner
    }

    fn show_input_time_data_selector(
        &self,
        ui: &mut egui::Ui,
        buffer: &mut NodeInput<I>,
    ) -> egui::Response {
        ui.scope(|ui| {
            ui.set_min_width(35.);
            let (current_value, pin) = match buffer {
                NodeInput::Time(p) => (SelectionType::Time, p.clone()),
                NodeInput::Data(p, _) => (SelectionType::Data, p.clone()),
            };

            let mut selected = current_value;

            let mut response = PickerWidget::new_salted(NodeInputPin::from(&*buffer))
                .ui(ui, format!("{:?}", selected), |ui| {
                    let mut response =
                        ui.selectable_value(&mut selected, SelectionType::Time, "Time");
                    response |= ui.selectable_value(&mut selected, SelectionType::Data, "Data");

                    response
                })
                .response;

            if selected != current_value {
                response.mark_changed();
                match selected {
                    SelectionType::Time => {
                        *buffer = NodeInput::Time(pin);
                    }
                    SelectionType::Data => {
                        *buffer = NodeInput::Data(pin, DataSpec::default());
                    }
                }
            }

            response
        })
        .inner
    }

    fn show_input_time(
        &mut self,
        ui: &mut egui::Ui,
        show_i: impl Fn(&mut egui::Ui, &mut I) -> egui::Response,
        input: &mut I,
    ) -> egui::Response {
        let response = show_i(ui, input);

        response
    }

    fn show_input_data(
        &mut self,
        ui: &mut egui::Ui,
        show_i: impl Fn(&mut egui::Ui, &mut I) -> egui::Response,
        input: &mut I,
        spec: &mut DataSpec,
    ) -> egui::Response {
        let mut response = show_i(ui, input);
        response |= ui.add(DataSpecWidget::new_salted(spec, "data spec widget"));

        response
    }

    fn show_output(
        &mut self,
        ui: &mut egui::Ui,
        output: NodeOutput,
        index: usize,
    ) -> egui::Response {
        let output_key = NodeOutputPin::from(output.clone());

        let buffer_id = ui.id().with("output buffer").with(&output_key);
        let mut buffer = ui.memory_mut(|mem| {
            mem.data
                .get_temp_mut_or_insert_with(buffer_id, || output.clone())
                .clone()
        });

        ui.horizontal(|ui| {
            let mut response = self.item_controls(
                ui,
                index,
                self.io_spec.len_output(),
                |this| {
                    this.io_spec.shift_output_index(&output_key, -1);
                },
                |this| {
                    this.io_spec.shift_output_index(&output_key, 1);
                },
                |this| {
                    this.io_spec.remove_output(&output_key);
                },
            );

            response |= self.show_output_time_data_selector(ui, &mut buffer);
            match &mut buffer {
                NodeOutput::Time => {}
                NodeOutput::Data(output, data_spec) => {
                    response |= self.show_output_data(ui, output, data_spec);
                }
            };

            ui.memory_mut(|mem| mem.data.insert_temp(buffer_id, buffer.clone()));

            if response.changed() && self.io_spec.update_output(&output_key, buffer.clone()) {
                ui.memory_mut(|mem| mem.data.remove_temp::<NodeOutput>(buffer_id));
            }

            response
        })
        .inner
    }

    fn show_output_time_data_selector(
        &self,
        ui: &mut egui::Ui,
        buffer: &mut NodeOutput,
    ) -> egui::Response {
        ui.scope(|ui| {
            ui.set_min_width(35.);
            let (current_value, pin) = match buffer {
                NodeOutput::Time => (SelectionType::Time, None),
                NodeOutput::Data(p, _) => (SelectionType::Data, Some(p.clone())),
            };

            let mut selected = current_value;

            let mut response = PickerWidget::new_salted(NodeOutputPin::from(&*buffer))
                .ui(ui, format!("{:?}", selected), |ui| {
                    let mut response =
                        ui.selectable_value(&mut selected, SelectionType::Time, "Time");
                    response |= ui.selectable_value(&mut selected, SelectionType::Data, "Data");

                    response
                })
                .response;

            if selected != current_value {
                response.mark_changed();
                match selected {
                    SelectionType::Time => {
                        *buffer = NodeOutput::Time;
                    }
                    SelectionType::Data => {
                        *buffer = NodeOutput::Data(pin.unwrap_or("".into()), DataSpec::default());
                    }
                }
            }

            response
        })
        .inner
    }

    fn show_output_data(
        &mut self,
        ui: &mut egui::Ui,
        input: &mut PinId,
        spec: &mut DataSpec,
    ) -> egui::Response {
        let mut response = ui.add(egui::TextEdit::singleline(input).desired_width(100.));
        response |= ui.add(DataSpecWidget::new_salted(spec, "data spec widget"));

        response
    }

    fn item_controls(
        &mut self,
        ui: &mut egui::Ui,
        i: usize,
        size: usize,
        move_up_callback: impl FnOnce(&mut Self),
        move_down_callback: impl FnOnce(&mut Self),
        delete_callback: impl FnOnce(&mut Self),
    ) -> egui::Response {
        ui.scope(|ui| {
            ui.set_min_width(60.);
            let mut move_up = None;
            let mut move_down = None;
            let mut delete = None;

            let button =
                |ui: &mut egui::Ui, text: &str| ui.add(egui::Button::new(text).frame(false));

            let mut response = button(ui, "🗙");
            if response.clicked() {
                delete = Some(i);
            }

            let up_response = ui.add_enabled_ui(i > 0, |ui| button(ui, "⬆")).inner;
            if i > 0 && up_response.clicked() {
                move_up = Some(i);
            }
            response |= up_response;

            let down_response = ui
                .add_enabled_ui(i < (size - 1), |ui| button(ui, "⬇"))
                .inner;
            if i < size - 1 && down_response.clicked() {
                move_down = Some(i);
            }
            response |= down_response;

            if let Some(_) = move_up {
                response.mark_changed();
                move_up_callback(self);
            }

            if let Some(_) = move_down {
                response.mark_changed();
                move_down_callback(self);
            }

            if let Some(_) = delete {
                response.mark_changed();
                delete_callback(self);
            }

            response
        })
        .inner
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum SelectionType {
    Time,
    Data,
}
