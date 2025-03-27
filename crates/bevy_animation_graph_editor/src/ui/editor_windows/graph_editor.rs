use bevy::{
    asset::Assets,
    input::ButtonInput,
    prelude::{KeyCode, World},
};
use bevy_animation_graph::{
    core::state_machine::high_level::StateMachine,
    prelude::{AnimationGraph, SpecContext},
};
use egui_dock::egui;

use crate::{
    graph_show::GraphReprSpec,
    graph_update::convert_graph_change,
    ui::{
        actions::{
            saving::{RequestSaveGraph, SaveAction},
            EditorAction,
        },
        core::{EditorWindowContext, EditorWindowExtension, InspectorSelection, NodeSelection},
        utils,
    },
};

#[derive(Debug)]
pub struct GraphEditorWindow;

impl EditorWindowExtension for GraphEditorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let Some(graph_selection) = &mut ctx.global_state.graph_editor else {
            ui.centered_and_justified(|ui| ui.label("Select a graph to edit!"));
            return;
        };

        world.resource_scope::<Assets<AnimationGraph>, _>(|world, mut graph_assets| {
            world.resource_scope::<Assets<StateMachine>, _>(|world, fsm_assets| {
                if !graph_assets.contains(graph_selection.graph) {
                    return;
                }

                let changes = {
                    let graph = graph_assets.get(graph_selection.graph).unwrap();
                    let spec_context = SpecContext {
                        graph_assets: &graph_assets,
                        fsm_assets: &fsm_assets,
                    };

                    // Autoselect context if none selected and some available
                    if let (Some(scene), Some(available_contexts)) = (
                        &mut ctx.global_state.scene,
                        utils::list_graph_contexts(world, |ctx| {
                            ctx.get_graph_id() == graph_selection.graph
                        }),
                    ) {
                        if scene
                            .active_context
                            .get(&graph_selection.graph.untyped())
                            .is_none()
                            && !available_contexts.is_empty()
                        {
                            scene
                                .active_context
                                .insert(graph_selection.graph.untyped(), available_contexts[0]);
                        }
                    }

                    let graph_player = utils::get_animation_graph_player(world);

                    let maybe_graph_context = ctx
                        .global_state
                        .scene
                        .as_ref()
                        .and_then(|s| s.active_context.get(&graph_selection.graph.untyped()))
                        .zip(graph_player)
                        .and_then(|(id, p)| Some(id).zip(p.get_context_arena()))
                        .and_then(|(id, ca)| ca.get_context(*id));

                    let nodes = GraphReprSpec::from_graph(
                        graph,
                        &graph_selection.graph_indices,
                        spec_context,
                        maybe_graph_context,
                    );

                    graph_selection
                        .nodes_context
                        .show(nodes.nodes, nodes.edges, ui);
                    graph_selection.nodes_context.get_changes().clone()
                }
                .into_iter()
                .map(|c| {
                    convert_graph_change(c, &graph_selection.graph_indices, graph_selection.graph)
                });
                ctx.graph_changes.extend(changes);

                // --- Update selection for node inspector.
                // --- And enable debug render for latest node selected only
                // ----------------------------------------------------------------

                let graph = graph_assets.get_mut(graph_selection.graph).unwrap();
                for (_, node) in graph.nodes.iter_mut() {
                    node.should_debug = false;
                }
                if let Some(selected_node) = graph_selection
                    .nodes_context
                    .get_selected_nodes()
                    .iter()
                    .rev()
                    .find(|id| **id > 1)
                {
                    if *selected_node > 1 {
                        let node_name = graph_selection
                            .graph_indices
                            .node_indices
                            .name(*selected_node)
                            .unwrap();
                        graph.nodes.get_mut(node_name).unwrap().should_debug = true;
                        if let InspectorSelection::Node(node_selection) =
                            &mut ctx.global_state.inspector_selection
                        {
                            if &node_selection.node != node_name
                                || node_selection.graph != graph_selection.graph
                            {
                                node_selection.node.clone_from(node_name);
                                node_selection.name_buf.clone_from(node_name);
                                node_selection.graph = graph_selection.graph;
                            }
                        } else if graph_selection.nodes_context.is_node_just_selected() {
                            ctx.global_state.inspector_selection =
                                InspectorSelection::Node(NodeSelection {
                                    graph: graph_selection.graph,
                                    node: node_name.clone(),
                                    name_buf: node_name.clone(),
                                    selected_pin_id: None,
                                });
                        }
                    }
                }
                // ----------------------------------------------------------------
            });
        });

        // --- Initiate graph saving if Ctrl+S pressed
        // ----------------------------------------------------------------
        world.resource_scope::<ButtonInput<KeyCode>, ()>(|_, input| {
            if input.pressed(KeyCode::ControlLeft) && input.just_pressed(KeyCode::KeyS) {
                ctx.editor_actions
                    .push(EditorAction::Save(SaveAction::RequestGraph(
                        RequestSaveGraph {
                            graph: graph_selection.graph,
                        },
                    )));
            }
        });
        // ----------------------------------------------------------------
    }

    fn display_name(&self) -> String {
        "Graph Editor".to_string()
    }
}
