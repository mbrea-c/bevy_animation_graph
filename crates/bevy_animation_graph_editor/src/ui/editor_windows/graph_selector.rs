use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::world::CommandQueue,
    log::info,
    prelude::World,
};
use bevy_animation_graph::prelude::AnimationGraph;
use egui_dock::egui;

use crate::{
    egui_nodes::lib::NodesContext,
    tree::TreeResult,
    ui::{
        actions::saving::DirtyAssets,
        core::{EditorWindowContext, EditorWindowExtension, GraphSelection, InspectorSelection},
        utils,
    },
};

#[derive(Debug)]
pub struct GraphSelectorWindow;

impl EditorWindowExtension for GraphSelectorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let mut queue = CommandQueue::default();
        let mut chosen_handle: Option<Handle<AnimationGraph>> = None;

        world.resource_scope::<AssetServer, ()>(|world, asset_server| {
            world.resource_scope::<Assets<AnimationGraph>, ()>(|world, mut graph_assets| {
                let mut assets: Vec<_> = graph_assets.ids().collect();
                assets.sort();
                let paths = assets
                    .into_iter()
                    .map(|id| (utils::handle_path(id.untyped(), &asset_server), id))
                    .collect();
                if let TreeResult::Leaf(id) = utils::path_selector(ui, paths) {
                    chosen_handle = Some(graph_assets.get_strong_handle(id).unwrap());
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    let mut dirty_assets = world.get_resource_mut::<DirtyAssets>().unwrap();
                    if ui.button("New Graph").clicked() {
                        let new_handle = graph_assets.add(AnimationGraph::default());
                        info!("Creating graph with id: {:?}", new_handle.id());
                        dirty_assets
                            .assets
                            .insert(new_handle.id().untyped(), new_handle.untyped());
                    }
                });
            });
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
