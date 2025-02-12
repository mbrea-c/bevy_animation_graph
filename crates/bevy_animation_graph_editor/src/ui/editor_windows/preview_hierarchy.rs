use bevy::prelude::World;
use bevy_animation_graph::{core::animation_clip::EntityPath, prelude::AnimatedSceneInstance};
use egui_dock::egui;

use crate::{
    tree::{Tree, TreeResult},
    ui::{
        core::{EditorContext, EditorWindowExtension},
        utils, PreviewScene,
    },
};

#[derive(Debug)]
pub struct PreviewHierarchyWindow;

impl EditorWindowExtension for PreviewHierarchyWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorContext) {
        if ctx.selection.scene.is_none() {
            return;
        };
        let mut query = world.query::<(&AnimatedSceneInstance, &PreviewScene)>();
        let Ok((instance, _)) = query.get_single(world) else {
            return;
        };
        let entity = instance.player_entity;
        let tree = Tree::entity_tree(world, entity);
        match utils::select_from_branches(ui, tree.0) {
            TreeResult::Leaf((_, path)) => {
                let path = EntityPath { parts: path };
                ui.output_mut(|o| o.copied_text = path.to_slashed_string());
                ctx.notifications
                    .info(format!("{} copied to clipboard", path.to_slashed_string()));
                ctx.selection.entity_path = Some(path);
            }
            TreeResult::Node((_, path)) => {
                let path = EntityPath { parts: path };
                ui.output_mut(|o| o.copied_text = path.to_slashed_string());
                ctx.notifications
                    .info(format!("{} copied to clipboard", path.to_slashed_string()));
                ctx.selection.entity_path = Some(path);
            }
            TreeResult::None => (),
        }
    }

    fn display_name(&self) -> String {
        "Preview Hierarchy".to_string()
    }
}
