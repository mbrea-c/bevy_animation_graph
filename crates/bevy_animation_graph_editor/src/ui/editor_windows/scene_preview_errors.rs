use bevy::prelude::World;
use bevy_animation_graph::prelude::{AnimatedSceneInstance, AnimationGraphPlayer};
use egui_dock::egui;

use crate::ui::{
    core::{EditorWindowContext, EditorWindowExtension},
    PreviewScene,
};

#[derive(Debug)]
pub struct ScenePreviewErrorsWindow;

impl EditorWindowExtension for ScenePreviewErrorsWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        if ctx.global_state.scene.is_none() {
            return;
        };
        let mut query = world.query::<(&AnimatedSceneInstance, &PreviewScene)>();
        let Ok((instance, _)) = query.get_single(world) else {
            return;
        };
        let entity = instance.player_entity();
        let mut query = world.query::<&AnimationGraphPlayer>();
        let Ok(player) = query.get(world, entity) else {
            return;
        };
        if let Some(error) = player.get_error() {
            ui.horizontal(|ui| {
                ui.label("⚠");
                ui.label(format!("{}", error));
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No errors to show");
            });
        }
    }

    fn display_name(&self) -> String {
        "Errors".to_string()
    }
}
