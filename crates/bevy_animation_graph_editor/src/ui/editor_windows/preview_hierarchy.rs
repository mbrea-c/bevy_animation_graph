use bevy::prelude::World;
use bevy_animation_graph::{core::animation_clip::EntityPath, prelude::AnimatedSceneInstance};
use egui_dock::egui;

use crate::{
    tree::{Tree, TreeResult},
    ui::{
        PreviewScene,
        core::{EditorWindowContext, EditorWindowExtension},
        utils,
    },
};

#[derive(Debug)]
pub struct PreviewHierarchyWindow;

impl EditorWindowExtension for PreviewHierarchyWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        if ctx.global_state.scene.is_none() {
            return;
        };
        let mut query = world.query::<(&AnimatedSceneInstance, &PreviewScene)>();
        let Ok((instance, _)) = query.single(world) else {
            return;
        };
        let entity = instance.player_entity();
        let tree = Tree::entity_tree(world, entity);
        match utils::select_from_branches(ui, tree.0) {
            TreeResult::Leaf((_, path), _) => {
                let path = EntityPath { parts: path };
                ui.output_mut(|o| {
                    o.commands
                        .push(egui::OutputCommand::CopyText(path.to_slashed_string()))
                });
                ctx.notifications
                    .info(format!("{} copied to clipboard", path.to_slashed_string()));
                ctx.global_state.entity_path = Some(path);
            }
            TreeResult::Node((_, path), _) => {
                let path = EntityPath { parts: path };
                ui.output_mut(|o| {
                    o.commands
                        .push(egui::OutputCommand::CopyText(path.to_slashed_string()))
                });
                ctx.notifications
                    .info(format!("{} copied to clipboard", path.to_slashed_string()));
                ctx.global_state.entity_path = Some(path);
            }
            TreeResult::None => (),
        }
    }

    fn display_name(&self) -> String {
        "Preview Hierarchy".to_string()
    }
}
