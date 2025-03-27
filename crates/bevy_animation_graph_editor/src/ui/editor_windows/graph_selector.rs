use bevy::{
    asset::{AssetId, AssetServer, Assets},
    ecs::world::CommandQueue,
    log::info,
    prelude::World,
};
use bevy_animation_graph::prelude::AnimationGraph;
use egui_dock::egui;

use crate::{
    egui_nodes::lib::NodesContext,
    scanner::PersistedAssetHandles,
    tree::TreeResult,
    ui::{
        core::{EditorWindowContext, EditorWindowExtension, GraphSelection, InspectorSelection},
        utils,
    },
};

#[derive(Debug)]
pub struct GraphSelectorWindow;

impl EditorWindowExtension for GraphSelectorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let mut queue = CommandQueue::default();
        let mut chosen_id: Option<AssetId<AnimationGraph>> = None;

        world.resource_scope::<AssetServer, ()>(|world, asset_server| {
            world.resource_scope::<Assets<AnimationGraph>, ()>(|world, mut graph_assets| {
                let mut assets: Vec<_> = graph_assets.ids().collect();
                assets.sort();
                let paths = assets
                    .into_iter()
                    .map(|id| (utils::handle_path(id.untyped(), &asset_server), id))
                    .collect();
                if let TreeResult::Leaf(id) = utils::path_selector(ui, paths) {
                    chosen_id = Some(id);
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    let mut graph_handles =
                        world.get_resource_mut::<PersistedAssetHandles>().unwrap();
                    if ui.button("New Graph").clicked() {
                        let new_handle = graph_assets.add(AnimationGraph::default());
                        info!("Creating graph with id: {:?}", new_handle.id());
                        graph_handles.unsaved_graphs.insert(new_handle);
                    }
                });
            });
        });
        queue.apply(world);
        if let Some(chosen_id) = chosen_id {
            ctx.global_state.graph_editor = Some(GraphSelection {
                graph: chosen_id,
                graph_indices: utils::update_graph_indices(world, chosen_id),
                nodes_context: NodesContext::default(),
            });
            ctx.global_state.inspector_selection = InspectorSelection::Graph;
        }
    }

    fn display_name(&self) -> String {
        "Select Graph".to_string()
    }
}
