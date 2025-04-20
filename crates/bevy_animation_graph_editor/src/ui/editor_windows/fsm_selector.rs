use bevy::{ecs::world::CommandQueue, prelude::World};
use bevy_animation_graph::core::state_machine::high_level::{State, StateMachine, Transition};
use egui_dock::egui;

use crate::{
    egui_fsm::lib::FsmUiContext,
    ui::{
        core::{EditorWindowContext, EditorWindowExtension, FsmSelection, InspectorSelection},
        utils::tree_asset_selector,
    },
};

#[derive(Debug)]
pub struct FsmSelectorWindow;

impl EditorWindowExtension for FsmSelectorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let mut queue = CommandQueue::default();
        let chosen_handle = tree_asset_selector::<StateMachine>(ui, world);

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
