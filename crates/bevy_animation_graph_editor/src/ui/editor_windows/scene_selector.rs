use bevy::{platform::collections::HashMap, prelude::World};
use bevy_animation_graph::{core::edge_data::AnimationEvent, prelude::AnimatedScene};
use egui_dock::egui;

use crate::ui::{
    core::{EditorWindowContext, EditorWindowExtension, SceneSelection},
    utils::tree_asset_selector,
};

#[derive(Debug)]
pub struct SceneSelectorWindow;

impl EditorWindowExtension for SceneSelectorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let chosen_handle = tree_asset_selector::<AnimatedScene>(ui, world);

        // TODO: Make sure to clear out all places that hold a graph context id
        //       when changing scene selection.
        if let Some(chosen_handle) = chosen_handle {
            let event_table = if let Some(scn) = &ctx.global_state.scene {
                scn.event_table.clone()
            } else {
                Vec::new()
            };
            ctx.global_state.scene = Some(SceneSelection {
                scene: chosen_handle,
                active_context: HashMap::default(),
                event_table,
                event_editor: AnimationEvent::default(),
            });
        }
    }

    fn display_name(&self) -> String {
        "Select Scene".to_string()
    }
}
