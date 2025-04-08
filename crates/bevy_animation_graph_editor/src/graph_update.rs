use crate::{
    egui_nodes::lib::GraphChange as EguiGraphChange,
    fsm_show::FsmIndices,
    graph_show::{GraphIndices, Pin},
    ui::actions::graph::{
        CreateLink, GraphAction, MoveInput, MoveNode, MoveOutput, RemoveLink, RemoveNode,
    },
};
use bevy::{
    asset::{AssetId, Assets, Handle},
    ecs::world::World,
    math::Vec2,
    reflect::Reflect,
};
use bevy_animation_graph::core::{
    animation_graph::{AnimationGraph, PinMap},
    edge_data::DataValue,
    state_machine::high_level::{State, StateId, StateMachine, Transition, TransitionId},
};

use super::egui_fsm::lib::EguiFsmChange;

#[derive(Debug, Clone)]
pub enum GlobalChange {
    FsmChange {
        asset_id: AssetId<StateMachine>,
        change: FsmChange,
    },
    Noop,
}

#[derive(Debug, Clone)]
pub enum FsmChange {
    StateMoved(StateId, Vec2),
    /// Some of the properties of a state are changed.
    /// This includes the state name.
    /// (original state id, new state)
    StateChanged(StateId, State),
    /// (original transition id, new transition)
    TransitionChanged(TransitionId, Transition),
    /// Add a new state.
    StateAdded(State),
    /// Add a new transition.
    TransitionAdded(Transition),
    /// Delete a state.
    StateDeleted(StateId),
    /// Delete a transition
    TransitionDeleted(TransitionId),
    /// Inspectable properties of the FSM must change.
    PropertiesChanged(FsmPropertiesChange),
}

#[derive(Debug, Clone, Reflect)]
pub struct FsmPropertiesChange {
    pub start_state: StateId,
    pub input_data: PinMap<DataValue>,
}

impl From<&StateMachine> for FsmPropertiesChange {
    fn from(value: &StateMachine) -> Self {
        Self {
            start_state: value.start_state.clone(),
            input_data: value.input_data.clone(),
        }
    }
}

pub fn convert_graph_change(
    graph_change: EguiGraphChange,
    graph_indices: &GraphIndices,
    graph_handle: Handle<AnimationGraph>,
) -> GraphAction {
    match graph_change {
        EguiGraphChange::LinkCreated(source_id, target_id) => {
            let source_pin = match graph_indices.pin_indices.pin(source_id).unwrap() {
                Pin::Source(s) => s,
                _ => panic!("Expected source pin"),
            }
            .clone();
            let target_pin = match graph_indices.pin_indices.pin(target_id).unwrap() {
                Pin::Target(t) => t,
                _ => panic!("Expected target pin"),
            }
            .clone();
            GraphAction::CreateLink(CreateLink {
                graph: graph_handle,
                source: source_pin,
                target: target_pin,
            })
        }
        EguiGraphChange::LinkRemoved(edge_id) => {
            let (_, target_id) = graph_indices.edge_indices.edge(edge_id).unwrap();
            let target_pin = match graph_indices.pin_indices.pin(*target_id).unwrap() {
                Pin::Target(t) => t,
                _ => panic!("Expected target pin"),
            }
            .clone();

            GraphAction::RemoveLink(RemoveLink {
                graph: graph_handle,
                target: target_pin,
            })
        }
        EguiGraphChange::NodeMoved(node_id, delta) => {
            if node_id == 0 {
                GraphAction::MoveInput(MoveInput {
                    graph: graph_handle,
                    new_pos: delta,
                })
            } else if node_id == 1 {
                GraphAction::MoveOutput(MoveOutput {
                    graph: graph_handle,
                    new_pos: delta,
                })
            } else {
                let node_id = graph_indices.node_indices.name(node_id).unwrap();
                GraphAction::MoveNode(MoveNode {
                    graph: graph_handle,
                    node: node_id.into(),
                    new_pos: delta,
                })
            }
        }
        EguiGraphChange::NodeRemoved(node_id) => {
            if node_id <= 1 {
                GraphAction::Noop
            } else {
                let node_id = graph_indices.node_indices.name(node_id).unwrap();
                GraphAction::RemoveNode(RemoveNode {
                    graph: graph_handle,
                    node: node_id.into(),
                })
            }
        }
    }
}

pub fn apply_global_changes(world: &mut World, changes: Vec<GlobalChange>) -> bool {
    let mut needs_regen_indices = false;

    for change in changes {
        needs_regen_indices = needs_regen_indices
            || match change {
                GlobalChange::FsmChange { asset_id, change } => {
                    apply_fsm_change(world, asset_id, change)
                }
                GlobalChange::Noop => false,
            };
    }

    needs_regen_indices
}

/// Apply a change to a state machine. Returns true if it requires recomputing the UI context, false otherwise.
fn apply_fsm_change(world: &mut World, asset_id: AssetId<StateMachine>, change: FsmChange) -> bool {
    let mut fsm_assets = world.resource_mut::<Assets<StateMachine>>();
    let fsm = fsm_assets.get_mut(asset_id).unwrap();

    match change {
        FsmChange::StateMoved(state_id, pos) => {
            fsm.extra.set_node_position(state_id, pos);
            false
        }
        FsmChange::StateChanged(old_state_name, new_state) => {
            let _ = fsm.update_state(old_state_name, new_state);
            true
        }
        FsmChange::TransitionChanged(old_transition_name, new_transition) => {
            let _ = fsm.update_transition(old_transition_name, new_transition);
            true
        }
        FsmChange::StateAdded(new_state) => {
            fsm.add_state(new_state);
            true
        }
        FsmChange::PropertiesChanged(new_props) => {
            fsm.set_start_state(new_props.start_state);
            fsm.set_input_data(new_props.input_data);
            true
        }
        FsmChange::TransitionAdded(new_transition) => {
            let _ = fsm.add_transition_from_ui(new_transition);
            true
        }
        FsmChange::StateDeleted(state_to_delete) => {
            let _ = fsm.delete_state(state_to_delete);
            true
        }
        FsmChange::TransitionDeleted(transition_to_delete) => {
            let _ = fsm.delete_transition(transition_to_delete);
            true
        }
    }
}

pub fn convert_fsm_change(
    fsm_change: EguiFsmChange,
    graph_indices: &FsmIndices,
    graph_id: AssetId<StateMachine>,
) -> GlobalChange {
    let change = match fsm_change {
        EguiFsmChange::StateMoved(state_id, delta) => {
            let node_id = graph_indices.state_indices.name(state_id).unwrap();
            GlobalChange::FsmChange {
                asset_id: graph_id,
                change: FsmChange::StateMoved(node_id.into(), delta),
            }
        }
        EguiFsmChange::TransitionRemoved(transition_id) => {
            let (_, transition_name, _) = graph_indices
                .transition_indices
                .edge(transition_id)
                .unwrap();
            GlobalChange::FsmChange {
                asset_id: graph_id,
                change: FsmChange::TransitionDeleted(transition_name.clone()),
            }
        }
        EguiFsmChange::StateRemoved(state_id) => {
            let state_name = graph_indices.state_indices.name(state_id).unwrap().clone();
            GlobalChange::FsmChange {
                asset_id: graph_id,
                change: FsmChange::StateDeleted(state_name),
            }
        }
        EguiFsmChange::TransitionCreated(_, _) => GlobalChange::Noop,
    };

    change
}
