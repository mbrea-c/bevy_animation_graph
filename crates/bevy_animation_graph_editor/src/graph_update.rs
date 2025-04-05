use crate::{
    egui_nodes::lib::GraphChange as EguiGraphChange,
    fsm_show::FsmIndices,
    graph_show::{GraphIndices, Pin},
};
use bevy::{
    asset::{AssetId, Assets, Handle},
    ecs::world::World,
    log::info,
    math::Vec2,
    reflect::Reflect,
};
use bevy_animation_graph::core::{
    animation_graph::{AnimationGraph, Edge, NodeId, PinMap, SourcePin, TargetPin},
    animation_node::AnimationNode,
    context::SpecContext,
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

#[derive(Debug, Clone)]
pub struct GraphChange {
    pub graph: Handle<AnimationGraph>,
    pub change: Change,
}

#[derive(Debug, Clone)]
pub enum Change {
    LinkCreated(SourcePin, TargetPin),
    LinkRemoved(TargetPin),
    NodeMoved(NodeId, Vec2),
    InputMoved(Vec2),
    OutputMoved(Vec2),
    /// (old_name, new_name)
    NodeRenamed(NodeId, String),
    NodeCreated(AnimationNode),
    NodeRemoved(NodeId),
    Noop,
    GraphValidate,
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
) -> GraphChange {
    let change = match graph_change {
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
            Change::LinkCreated(source_pin, target_pin)
        }
        EguiGraphChange::LinkRemoved(edge_id) => {
            let (_, target_id) = graph_indices.edge_indices.edge(edge_id).unwrap();
            let target_pin = match graph_indices.pin_indices.pin(*target_id).unwrap() {
                Pin::Target(t) => t,
                _ => panic!("Expected target pin"),
            }
            .clone();
            Change::LinkRemoved(target_pin)
        }
        EguiGraphChange::NodeMoved(node_id, delta) => {
            if node_id == 0 {
                Change::InputMoved(delta)
            } else if node_id == 1 {
                Change::OutputMoved(delta)
            } else {
                let node_id = graph_indices.node_indices.name(node_id).unwrap();
                Change::NodeMoved(node_id.into(), delta)
            }
        }
        EguiGraphChange::NodeRemoved(node_id) => {
            if node_id <= 1 {
                Change::Noop
            } else {
                let node_id = graph_indices.node_indices.name(node_id).unwrap();
                Change::NodeRemoved(node_id.into())
            }
        }
    };

    GraphChange {
        graph: graph_handle,
        change,
    }
}

pub fn update_graph_asset(
    mut changes: Vec<GraphChange>,
    graph_assets: &mut Assets<AnimationGraph>,
    fsm_assets: &Assets<StateMachine>,
) -> bool {
    let graph_assets_copy = unsafe { &*(graph_assets as *const Assets<AnimationGraph>) };
    let ctx = SpecContext {
        graph_assets: graph_assets_copy,
        fsm_assets,
    };
    let mut must_regen_indices = false;
    while let Some(change) = changes.pop() {
        let graph = graph_assets.get_mut(&change.graph).unwrap();
        match change.change {
            Change::LinkCreated(source_pin, target_pin) => {
                if let Ok(()) = graph.can_add_edge(
                    Edge {
                        source: source_pin.clone(),
                        target: target_pin.clone(),
                    },
                    ctx,
                ) {
                    info!("Adding edge {:?} -> {:?}", source_pin, target_pin);
                    graph.add_edge(source_pin, target_pin);
                    changes.push(GraphChange {
                        graph: change.graph,
                        change: Change::GraphValidate,
                    });
                }
            }
            Change::LinkRemoved(target_pin) => {
                info!("Removing edge with target {:?}", target_pin);
                graph.remove_edge_by_target(&target_pin);
                changes.push(GraphChange {
                    graph: change.graph,
                    change: Change::GraphValidate,
                });
            }
            Change::NodeMoved(node_id, new_pos) => {
                graph.extra.set_node_position(node_id, new_pos);
            }
            Change::InputMoved(new_pos) => {
                graph.extra.set_input_position(new_pos);
            }
            Change::OutputMoved(new_pos) => {
                graph.extra.set_output_position(new_pos);
            }
            Change::NodeRenamed(old_id, new_id) => {
                info!("Renaming node {:?} to {:?}", old_id, new_id);
                let _ = graph.rename_node(old_id, new_id);
                changes.push(GraphChange {
                    graph: change.graph,
                    change: Change::GraphValidate,
                });
            }
            Change::GraphValidate => {
                must_regen_indices = true;
                while let Err(deletable) = graph.validate_edges(ctx) {
                    for Edge { target, .. } in deletable {
                        info!("Removing edge with target {:?}", target);
                        graph.remove_edge_by_target(&target);
                    }
                }
            }
            Change::NodeCreated(animation_node) => {
                if !graph.nodes.contains_key(&animation_node.name) {
                    info!("Adding node {:?}", animation_node.name);
                    graph.add_node(animation_node);
                    changes.push(GraphChange {
                        graph: change.graph,
                        change: Change::GraphValidate,
                    });
                }
            }
            Change::NodeRemoved(node_id) => {
                info!("Removing node {:?}", node_id);
                graph.remove_node(node_id);
                changes.push(GraphChange {
                    graph: change.graph,
                    change: Change::GraphValidate,
                });
            }
            Change::Noop => {}
        }
    }
    must_regen_indices
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
