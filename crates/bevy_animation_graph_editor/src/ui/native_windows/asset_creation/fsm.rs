use bevy::prelude::World;
use bevy_animation_graph::core::state_machine::high_level::StateMachine;
use egui_dock::egui;

use crate::ui::{
    native_windows::{EditorWindowContext, NativeEditorWindowExtension},
    state_management::global::{CloseWindow, fsm::CreateFsm},
};

#[derive(Debug, Clone, Copy)]
pub struct CreateFsmWindow;

impl NativeEditorWindowExtension for CreateFsmWindow {
    fn ui(&self, ui: &mut egui::Ui, _: &mut World, ctx: &mut EditorWindowContext) {
        let buffer_id = ui.id().with("FSM Asset creator buffer");
        let fsm_path: &mut String = ctx.buffers.get_mut_or_default(buffer_id);

        ui.label("Asset path");
        ui.text_edit_singleline(fsm_path);

        let path = fsm_path.to_string();

        ui.add_enabled_ui(validate_path(fsm_path), |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                if ui.button("Create").clicked() {
                    ctx.trigger(CreateFsm {
                        fsm: StateMachine::default(),
                        virtual_path: path.to_string(),
                    });
                    ctx.trigger_window(CloseWindow::default());
                }
            });
        });
    }

    fn display_name(&self) -> String {
        "Create state machine".to_string()
    }
}

fn validate_path(path: &str) -> bool {
    path.ends_with(".fsm.ron")
}
