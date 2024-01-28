use crate::{
    egui_nodes::lib::GraphChange as EguiGraphChange,
    graph_show::{GraphIndices, Pin},
};
use bevy::{
    asset::{AssetId, Assets},
    log::info,
    math::Vec2,
};
use bevy_animation_graph::core::{
    animation_graph::{AnimationGraph, Edge, NodeId, SourcePin, TargetPin},
    animation_node::AnimationNode,
    context::SpecContext,
};

#[derive(Debug, Clone)]
pub struct GraphChange {
    pub graph: AssetId<AnimationGraph>,
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

pub fn convert_graph_change(
    graph_change: EguiGraphChange,
    graph_indices: &GraphIndices,
    graph_id: AssetId<AnimationGraph>,
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
        graph: graph_id,
        change,
    }
}

pub fn update_graph(
    mut changes: Vec<GraphChange>,
    graph_assets: &mut Assets<AnimationGraph>,
) -> bool {
    let graph_assets_copy = unsafe { &*(graph_assets as *const Assets<AnimationGraph>) };
    let ctx = SpecContext {
        graph_assets: graph_assets_copy,
    };
    let mut must_regen_indices = false;
    while let Some(change) = changes.pop() {
        let graph = graph_assets.get_mut(change.graph).unwrap();
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
                graph.remove_edge(&target_pin);
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
                        graph.remove_edge(&target);
                    }
                }
            }
            Change::NodeCreated(animation_node) => {
                info!("Adding node {:?}", animation_node.name);
                graph.add_node(animation_node);
                changes.push(GraphChange {
                    graph: change.graph,
                    change: Change::GraphValidate,
                });
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
