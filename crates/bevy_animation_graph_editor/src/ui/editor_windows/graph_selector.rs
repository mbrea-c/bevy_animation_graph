use bevy::{ecs::world::CommandQueue, prelude::World};
use bevy_animation_graph::prelude::AnimationGraph;
use egui_dock::egui;

use crate::{
    egui_nodes::lib::NodesContext,
    ui::{
        actions::graph::CreateGraphAction,
        core::{
            EditorWindowExtension, GraphSelection, InspectorSelection, LegacyEditorWindowContext,
        },
        utils::tree_asset_selector,
    },
};

#[derive(Debug)]
pub struct GraphSelectorWindow;

impl EditorWindowExtension for GraphSelectorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut LegacyEditorWindowContext) {
        let mut queue = CommandQueue::default();
        let chosen_handle = tree_asset_selector::<AnimationGraph>(ui, world);

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            if ui.button("New Graph").clicked() {
                ctx.editor_actions.dynamic(CreateGraphAction);
            }
        });

        queue.apply(world);
        if let Some(chosen_id) = chosen_handle {
            ctx.global_state.graph_editor = Some(GraphSelection {
                graph: chosen_id.clone(),
                nodes_context: NodesContext::default(),
            });
            ctx.global_state.inspector_selection = InspectorSelection::Graph;
        }
    }

    fn display_name(&self) -> String {
        "Select Graph".to_string()
    }
}
