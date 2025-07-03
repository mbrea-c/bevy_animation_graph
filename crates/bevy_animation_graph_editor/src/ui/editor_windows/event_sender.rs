use bevy::prelude::World;
use egui_dock::egui;

use crate::ui::{
    core::{EditorWindowContext, EditorWindowExtension},
    utils,
};

#[derive(Debug)]
pub struct EventSenderWindow;

impl EditorWindowExtension for EventSenderWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let Some(scene_selection) = &mut ctx.global_state.scene else {
            return;
        };

        let Some(graph_player) = utils::get_animation_graph_player_mut(world) else {
            return;
        };

        ui.horizontal_wrapped(|ui| {
            scene_selection.event_table.retain(|ev| {
                egui::Frame::NONE
                    .stroke(egui::Stroke::new(1., egui::Color32::WHITE))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button(format!("{ev:?}")).clicked() {
                                graph_player.send_event(ev.clone());
                            }
                            !ui.button("×").clicked()
                        })
                        .inner
                    })
                    .inner
            });
        });

        ui.separator();

        utils::using_inspector_env(world, |mut env| {
            env.ui_for_reflect(&mut scene_selection.event_editor, ui);
        });

        if ui.button("Add").clicked() {
            scene_selection
                .event_table
                .push(scene_selection.event_editor.clone());
        }
    }

    fn display_name(&self) -> String {
        "Send events".to_string()
    }
}
