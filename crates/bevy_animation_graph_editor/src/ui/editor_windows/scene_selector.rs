use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::world::CommandQueue,
    prelude::World,
    utils::HashMap,
};
use bevy_animation_graph::{core::edge_data::AnimationEvent, prelude::AnimatedScene};
use egui_dock::egui;

use crate::{
    tree::TreeResult,
    ui::{
        core::{EditorWindowContext, EditorWindowExtension, SceneSelection},
        utils,
    },
};

#[derive(Debug)]
pub struct SceneSelectorWindow;

impl EditorWindowExtension for SceneSelectorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let mut queue = CommandQueue::default();

        let mut chosen_handle: Option<Handle<AnimatedScene>> = None;

        world.resource_scope::<AssetServer, ()>(|world, asset_server| {
            // create a context with access to the world except for the `R` resource
            world.resource_scope::<Assets<AnimatedScene>, ()>(|_, assets| {
                let mut assets: Vec<_> = assets.ids().collect();
                assets.sort();
                let paths = assets
                    .into_iter()
                    .map(|id| (utils::handle_path(id.untyped(), &asset_server), id))
                    .collect();
                let chosen_id = utils::path_selector(ui, paths);
                if let TreeResult::Leaf(id) = chosen_id {
                    chosen_handle = Some(
                        asset_server
                            .get_handle(asset_server.get_path(id).unwrap())
                            .unwrap(),
                    )
                }
            });
        });
        queue.apply(world);

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
