use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::world::CommandQueue,
    prelude::World,
};
use bevy_animation_graph::core::state_machine::high_level::{State, StateMachine, Transition};
use egui_dock::egui;

use crate::{
    egui_fsm::lib::FsmUiContext,
    tree::TreeResult,
    ui::{
        core::{EditorWindowContext, EditorWindowExtension, FsmSelection, InspectorSelection},
        utils,
    },
};

#[derive(Debug)]
pub struct FsmSelectorWindow;

impl EditorWindowExtension for FsmSelectorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let mut queue = CommandQueue::default();
        let mut chosen_handle: Option<Handle<StateMachine>> = None;

        world.resource_scope::<AssetServer, ()>(|world, asset_server| {
            world.resource_scope::<Assets<StateMachine>, ()>(|_world, mut graph_assets| {
                let mut assets: Vec<_> = graph_assets.ids().collect();
                assets.sort();
                let paths = assets
                    .into_iter()
                    .map(|id| (utils::handle_path(id.untyped(), &asset_server), id))
                    .collect();
                if let TreeResult::Leaf(id) = utils::path_selector(ui, paths) {
                    chosen_handle = Some(graph_assets.get_strong_handle(id).unwrap());
                }
                // ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                //     let mut graph_handles = world.get_resource_mut::<GraphHandles>().unwrap();
                //     CREATE NEW FSM & STUFF
                // });
            });
        });
        queue.apply(world);
        if let Some(chosen_id) = chosen_handle {
            ctx.global_state.fsm_editor = Some(FsmSelection {
                fsm: chosen_id,
                nodes_context: FsmUiContext::default(),
                state_creation: State::default(),
                transition_creation: Transition::default(),
            });
            ctx.global_state.inspector_selection = InspectorSelection::Fsm;
        }
    }

    fn display_name(&self) -> String {
        "Select FSM".to_string()
    }
}
