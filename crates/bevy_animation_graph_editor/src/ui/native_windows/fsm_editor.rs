use bevy::{
    asset::{AssetId, Assets, Handle},
    prelude::World,
    utils::default,
};
use bevy_animation_graph::{
    core::state_machine::{high_level::StateMachine, low_level::FSMState},
    prelude::{AnimationGraph, node_states::StateKey},
};
use egui_dock::egui;

use crate::{
    egui_fsm::lib::{EguiFsmChange, FsmUiContext},
    fsm_show::{FsmIndices, FsmIndicesMap, FsmReprSpec},
    ui::{
        actions::{
            EditorAction,
            fsm::{FsmAction, GenerateIndices, MoveState, RemoveState, RemoveTransition},
        },
        global_state::{
            active_fsm::ActiveFsm,
            active_fsm_state::{ActiveFsmState, SetActiveFsmState},
            active_fsm_transition::{ActiveFsmTransition, SetActiveFsmTransition},
            active_graph_context::ActiveContexts,
            get_global_state,
            inspector_selection::{InspectorSelection, SetInspectorSelection},
        },
        native_windows::{EditorWindowContext, NativeEditorWindowExtension},
        utils,
    },
};

#[derive(Default)]
pub struct FsmEditBuffer {
    pub fsm: AssetId<StateMachine>,
    pub context: FsmUiContext,
}

#[derive(Debug)]
pub struct FsmEditorWindow;

impl NativeEditorWindowExtension for FsmEditorWindow {
    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let Some(active_fsm) = get_global_state::<ActiveFsm>(world).cloned() else {
            ui.centered_and_justified(|ui| ui.label("Select a state machine to edit!"));
            return;
        };

        let mut queue = ctx.make_queue();

        let buffer_id = ui.id().with("Fsm editor nodes context buffer");
        let buffer = ctx
            .buffers
            .get_mut_or_insert_with(buffer_id, || FsmEditBuffer {
                fsm: active_fsm.handle.id(),
                ..default()
            });
        let buffer = if buffer.fsm != active_fsm.handle.id() {
            ctx.buffers.clear::<FsmEditBuffer>(buffer_id);
            ctx.buffers
                .get_mut_or_insert_with(buffer_id, || FsmEditBuffer {
                    fsm: active_fsm.handle.id(),
                    ..default()
                })
        } else {
            buffer
        };

        world.resource_scope::<Assets<StateMachine>, _>(|world, fsm_assets| {
            world.resource_scope::<Assets<AnimationGraph>, _>(|world, graph_assets| {
                world.resource_scope::<FsmIndicesMap, _>(|world, fsm_indices_map| {
                    let fsm = fsm_assets.get(&active_fsm.handle)?;

                    let Some(fsm_indices) = fsm_indices_map.indices.get(&active_fsm.handle.id())
                    else {
                        ctx.editor_actions
                            .push(EditorAction::Fsm(FsmAction::GenerateIndices(
                                GenerateIndices {
                                    fsm: active_fsm.handle.id(),
                                },
                            )));

                        return None;
                    };

                    {
                        let maybe_graph_context = get_global_state::<ActiveContexts>(world)
                            .and_then(|s| s.by_asset.get(&active_fsm.handle.id().untyped()))
                            .and_then(|(entity, id)| {
                                Some((
                                    id,
                                    utils::get_specific_animation_graph_player(world, *entity)?,
                                ))
                            })
                            .and_then(|(id, p)| Some(id).zip(p.get_context_arena()))
                            .and_then(|(id, ca)| ca.get_context(*id));

                        let maybe_fsm_state = maybe_graph_context.and_then(|ctx| {
                            let graph_id = ctx.get_graph_id();
                            let graph = graph_assets.get(graph_id)?;
                            let node_id = graph.contains_state_machine(&active_fsm.handle)?;
                            ctx.node_states
                                .get::<FSMState>(node_id, StateKey::Default)
                                .ok()
                                .cloned()
                        });

                        let fsm_repr_spec =
                            FsmReprSpec::from_fsm(fsm, fsm_indices, &fsm_assets, maybe_fsm_state);

                        buffer
                            .context
                            .show(fsm_repr_spec.states, fsm_repr_spec.transitions, ui);
                        buffer.context.get_changes().clone()
                    }
                    .into_iter()
                    .filter_map(|c| convert_fsm_change(c, fsm_indices, active_fsm.handle.clone()))
                    .for_each(|action| ctx.editor_actions.push(EditorAction::Fsm(action)));

                    // --- Update selection for state inspector.
                    // ----------------------------------------------------------------

                    if let Some(selected_node) = buffer
                        .context
                        .get_selected_states()
                        .iter()
                        .rev()
                        .find(|id| **id > 1)
                    {
                        let state_name = fsm_indices.state_indices.name(*selected_node).unwrap();
                        if buffer.context.is_node_just_selected() {
                            queue.trigger(SetActiveFsmState {
                                new: ActiveFsmState {
                                    handle: active_fsm.handle.clone(),
                                    state: state_name.clone(),
                                },
                            });
                            queue.trigger(SetInspectorSelection {
                                selection: InspectorSelection::ActiveFsmState,
                            });
                        }
                    }

                    if let Some(selected_transition) =
                        buffer.context.get_selected_transitions().iter().next_back()
                    {
                        let (_, transition_id, _) = fsm_indices
                            .transition_indices
                            .edge(*selected_transition)
                            .unwrap();
                        if buffer.context.is_transition_just_selected() {
                            queue.trigger(SetActiveFsmTransition {
                                new: ActiveFsmTransition {
                                    handle: active_fsm.handle.clone(),
                                    transition: transition_id.clone(),
                                },
                            });
                            queue.trigger(SetInspectorSelection {
                                selection: InspectorSelection::ActiveFsmTransition,
                            });
                        }
                    }
                    // ----------------------------------------------------------------
                    Some(())
                })
            })
        });

        ctx.consume_queue(queue);
    }

    fn display_name(&self) -> String {
        "FSM Editor".to_string()
    }
}

pub fn convert_fsm_change(
    fsm_change: EguiFsmChange,
    fsm_indices: &FsmIndices,
    fsm: Handle<StateMachine>,
) -> Option<FsmAction> {
    match fsm_change {
        EguiFsmChange::StateMoved(state_id, delta) => {
            let node_id = fsm_indices.state_indices.name(state_id).unwrap();
            Some(FsmAction::MoveState(MoveState {
                fsm,
                state_id: node_id.into(),
                new_pos: delta,
            }))
        }
        EguiFsmChange::TransitionRemoved(transition_id) => {
            let (_, transition_name, _) =
                fsm_indices.transition_indices.edge(transition_id).unwrap();
            Some(FsmAction::RemoveTransition(RemoveTransition {
                fsm,
                transition_id: transition_name.clone(),
            }))
        }
        EguiFsmChange::StateRemoved(state_id) => {
            let state_name = fsm_indices.state_indices.name(state_id).unwrap().clone();

            Some(FsmAction::RemoveState(RemoveState {
                fsm,
                state_id: state_name,
            }))
        }
        EguiFsmChange::TransitionCreated(_, _) => None,
    }
}
