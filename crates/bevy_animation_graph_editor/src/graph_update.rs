use crate::{
    egui_nodes::lib::GraphChange as EguiGraphChange,
    fsm_show::FsmIndices,
    graph_show::{GraphIndices, Pin},
    ui::actions::{
        fsm::{FsmAction, MoveState, RemoveState, RemoveTransition},
        graph::{CreateLink, GraphAction, MoveInput, MoveNode, MoveOutput, RemoveLink, RemoveNode},
    },
};
use bevy::asset::Handle;
use bevy_animation_graph::core::{
    animation_graph::AnimationGraph, state_machine::high_level::StateMachine,
};

use super::egui_fsm::lib::EguiFsmChange;

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

pub fn convert_fsm_change(
    fsm_change: EguiFsmChange,
    fsm_indices: &FsmIndices,
    fsm: Handle<StateMachine>,
) -> Option<FsmAction> {
    let change = match fsm_change {
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
    };

    change
}
