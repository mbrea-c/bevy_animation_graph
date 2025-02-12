use bevy::{
    ecs::world::CommandQueue,
    prelude::{AppTypeRegistry, World},
};
use bevy_inspector_egui::reflect_inspector::{Context, InspectorUi};
use egui_dock::egui;

use crate::ui::{
    core::{EditorContext, EditorWindowExtension},
    utils,
};

#[derive(Debug)]
pub struct EventSenderWindow;

impl EditorWindowExtension for EventSenderWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorContext) {
        let unsafe_world = world.as_unsafe_world_cell();
        let type_registry = unsafe {
            unsafe_world
                .get_resource::<AppTypeRegistry>()
                .unwrap()
                .0
                .clone()
        };

        let type_registry = type_registry.read();
        let mut queue = CommandQueue::default();
        let mut cx = Context {
            world: Some(unsafe { unsafe_world.world_mut() }.into()),
            queue: Some(&mut queue),
        };
        let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

        let Some(scene_selection) = &mut ctx.selection.scene else {
            return;
        };
        let Some(graph_player) =
            utils::get_animation_graph_player_mut(unsafe { unsafe_world.world_mut() })
        else {
            return;
        };

        ui.horizontal_wrapped(|ui| {
            scene_selection.event_table.retain(|ev| {
                egui::Frame::none()
                    .stroke(egui::Stroke::new(1., egui::Color32::WHITE))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button(format!("{:?}", ev)).clicked() {
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

        env.ui_for_reflect(&mut scene_selection.event_editor, ui);
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
