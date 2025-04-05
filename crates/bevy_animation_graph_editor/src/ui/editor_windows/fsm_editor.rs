use bevy::{asset::Assets, prelude::World};
use bevy_animation_graph::{
    core::state_machine::high_level::StateMachine, prelude::AnimationGraph,
};
use egui_dock::egui;

use crate::{
    fsm_show::FsmReprSpec,
    graph_update::convert_fsm_change,
    ui::{
        core::{
            EditorWindowContext, EditorWindowExtension, FsmStateSelection, FsmTransitionSelection,
            InspectorSelection,
        },
        utils,
    },
};

#[derive(Debug)]
pub struct FsmEditorWindow;

impl EditorWindowExtension for FsmEditorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let Some(fsm_selection) = &mut ctx.global_state.fsm_editor else {
            ui.centered_and_justified(|ui| ui.label("Select a state machine to edit!"));
            return;
        };

        world.resource_scope::<Assets<StateMachine>, ()>(|world, fsm_assets| {
            world.resource_scope::<Assets<AnimationGraph>, ()>(|world, graph_assets| {
                if !fsm_assets.contains(fsm_selection.fsm) {
                    return;
                }

                let changes = {
                    let fsm = fsm_assets.get(fsm_selection.fsm).unwrap();

                    // Autoselect context if none selected and some available
                    if let (Some(scene), Some(available_contexts)) = (
                        &mut ctx.global_state.scene,
                        utils::list_graph_contexts(world, |ctx| {
                            let graph_id = ctx.get_graph_id();
                            graph_assets
                                .get(graph_id)
                                .map(|graph| graph.contains_state_machine(fsm_selection.fsm))
                                .is_some()
                        }),
                    ) {
                        if scene
                            .active_context
                            .get(&fsm_selection.fsm.untyped())
                            .is_none()
                            && !available_contexts.is_empty()
                        {
                            scene
                                .active_context
                                .insert(fsm_selection.fsm.untyped(), available_contexts[0]);
                        }
                    }

                    let graph_player = utils::get_animation_graph_player(world);

                    let maybe_fsm_state = ctx
                        .global_state
                        .scene
                        .as_ref()
                        .and_then(|s| s.active_context.get(&fsm_selection.fsm.untyped()))
                        .zip(graph_player)
                        .and_then(|(id, p)| Some(id).zip(p.get_context_arena()))
                        .and_then(|(id, ca)| ca.get_context(*id))
                        .and_then(|ctx| {
                            let graph_id = ctx.get_graph_id();
                            let graph = graph_assets.get(graph_id).unwrap();
                            let node_id = graph.contains_state_machine(fsm_selection.fsm).unwrap();
                            ctx.caches
                                .get_primary(|c| c.get_fsm_state(&node_id).cloned())
                        });

                    let fsm_repr_spec = FsmReprSpec::from_fsm(
                        fsm,
                        &fsm_selection.graph_indices,
                        &fsm_assets,
                        maybe_fsm_state,
                    );

                    fsm_selection.nodes_context.show(
                        fsm_repr_spec.states,
                        fsm_repr_spec.transitions,
                        ui,
                    );
                    fsm_selection.nodes_context.get_changes().clone()
                }
                .into_iter()
                .map(|c| convert_fsm_change(c, &fsm_selection.graph_indices, fsm_selection.fsm));
                ctx.global_changes.extend(changes);

                // --- Update selection for state inspector.
                // ----------------------------------------------------------------

                if let Some(selected_node) = fsm_selection
                    .nodes_context
                    .get_selected_states()
                    .iter()
                    .rev()
                    .find(|id| **id > 1)
                {
                    let state_name = fsm_selection
                        .graph_indices
                        .state_indices
                        .name(*selected_node)
                        .unwrap();
                    if fsm_selection.nodes_context.is_node_just_selected() {
                        ctx.global_state.inspector_selection =
                            InspectorSelection::FsmState(FsmStateSelection {
                                fsm: fsm_selection.fsm,
                                state: state_name.clone(),
                            });
                    }
                }

                if let Some(selected_transition) = fsm_selection
                    .nodes_context
                    .get_selected_transitions()
                    .iter()
                    .next_back()
                {
                    let (_, transition_id, _) = fsm_selection
                        .graph_indices
                        .transition_indices
                        .edge(*selected_transition)
                        .unwrap();
                    if fsm_selection.nodes_context.is_transition_just_selected() {
                        ctx.global_state.inspector_selection =
                            InspectorSelection::FsmTransition(FsmTransitionSelection {
                                fsm: fsm_selection.fsm,
                                state: transition_id.clone(),
                            });
                    }
                }
                // ----------------------------------------------------------------
            });
        });
    }

    fn display_name(&self) -> String {
        "FSM Editor".to_string()
    }
}
