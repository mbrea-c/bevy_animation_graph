use bevy::prelude::World;
use bevy_animation_graph::core::edge_data::AnimationEvent;
use egui_dock::egui;

use crate::ui::{
    native_windows::{EditorWindowContext, NativeEditorWindowExtension},
    utils,
};

#[derive(Debug)]
pub struct EventSenderWindow;

impl NativeEditorWindowExtension for EventSenderWindow {
    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let Some(graph_player) = utils::get_animation_graph_player_mut(world) else {
            return;
        };

        let buffer = ctx
            .buffers
            .get_mut_or_default::<EventSenderBuffer>(ui.id().with("event sender table buffer"));

        ui.horizontal_wrapped(|ui| {
            buffer.events.retain(|ev| {
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
            env.ui_for_reflect(&mut buffer.event_editor, ui);
        });

        if ui.button("Add").clicked() {
            buffer.events.push(buffer.event_editor.clone());
        }
    }

    fn display_name(&self) -> String {
        "Send events".to_string()
    }
}

#[derive(Default)]
struct EventSenderBuffer {
    events: Vec<AnimationEvent>,
    event_editor: AnimationEvent,
}
