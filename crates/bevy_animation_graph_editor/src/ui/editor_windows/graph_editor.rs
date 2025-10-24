use std::any::TypeId;

use bevy::{
    asset::{Assets, Handle},
    ecs::reflect::AppTypeRegistry,
    prelude::World,
    reflect::{TypeRegistry, prelude::ReflectDefault},
};
use bevy_animation_graph::{
    core::state_machine::high_level::StateMachine,
    prelude::{AnimationGraph, ReflectNodeLike, SpecContext},
};
use egui_dock::egui;

use crate::{
    egui_nodes::lib::GraphChange as EguiGraphChange,
    graph_show::{GraphIndices, GraphIndicesMap, GraphReprSpec, Pin},
    ui::{
        actions::{
            EditorAction,
            graph::{
                CreateLink, CreateNode, GenerateIndices, GraphAction, MoveInput, MoveNode,
                MoveOutput, RemoveLink, RemoveNode,
            },
        },
        core::{
            EditorWindowExtension, InspectorSelection, LegacyEditorWindowContext, NodeSelection,
        },
        utils::{self, popup::CustomPopup, using_inspector_env},
    },
};

struct TypeInfo {
    id: TypeId,
    /// Full type path
    path: String,
    /// Last component of the type path (when separated on `::`)
    short: String,
}

#[derive(Debug)]
pub struct GraphEditorWindow;

impl EditorWindowExtension for GraphEditorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut LegacyEditorWindowContext) {
        let Some(graph_selection) = &mut ctx.global_state.graph_editor else {
            ui.centered_and_justified(|ui| ui.label("Select a graph to edit!"));
            return;
        };

        world.resource_scope::<Assets<AnimationGraph>, _>(|world, mut graph_assets| {
            world.resource_scope::<Assets<StateMachine>, _>(|world, fsm_assets| {
                world.resource_scope::<GraphIndicesMap, _>(|world, graph_indices_map| {
                    if !graph_assets.contains(&graph_selection.graph) {
                        return;
                    }

                    let Some(graph_indices) =
                        graph_indices_map.indices.get(&graph_selection.graph.id())
                    else {
                        ctx.editor_actions
                            .push(EditorAction::Graph(GraphAction::GenerateIndices(
                                GenerateIndices {
                                    graph: graph_selection.graph.id(),
                                },
                            )));
                        return;
                    };

                    {
                        let graph = graph_assets.get(&graph_selection.graph).unwrap();
                        let spec_context = SpecContext {
                            graph_assets: &graph_assets,
                            fsm_assets: &fsm_assets,
                        };

                        // Autoselect context if none selected and some available
                        if let Some(scene) = &mut ctx.global_state.scene
                            && let Some(available_contexts) =
                                utils::list_graph_contexts(world, |ctx| {
                                    ctx.get_graph_id() == graph_selection.graph.id()
                                })
                            && scene
                                .active_context
                                .get(&graph_selection.graph.id().untyped())
                                .is_none()
                            && !available_contexts.is_empty()
                        {
                            scene.active_context.insert(
                                graph_selection.graph.id().untyped(),
                                available_contexts[0],
                            );
                        }

                        let graph_player = utils::get_animation_graph_player(world);

                        let maybe_graph_context = ctx
                            .global_state
                            .scene
                            .as_ref()
                            .and_then(|s| {
                                s.active_context.get(&graph_selection.graph.id().untyped())
                            })
                            .zip(graph_player)
                            .and_then(|(id, p)| Some(id).zip(p.get_context_arena()))
                            .and_then(|(id, ca)| ca.get_context(*id));

                        let nodes = GraphReprSpec::from_graph(
                            graph,
                            graph_indices,
                            spec_context,
                            maybe_graph_context,
                        );

                        graph_selection
                            .nodes_context
                            .show(nodes.nodes, nodes.edges, ui);
                        graph_selection.nodes_context.get_changes().clone()
                    }
                    .into_iter()
                    .map(|c| convert_graph_change(c, graph_indices, graph_selection.graph.clone()))
                    .for_each(|action| ctx.editor_actions.push(EditorAction::Graph(action)));

                    // --- Update selection for node inspector.
                    // --- And enable debug render for latest node selected only
                    // ----------------------------------------------------------------

                    let graph = graph_assets.get_mut(&graph_selection.graph).unwrap();
                    for (_, node) in graph.nodes.iter_mut() {
                        node.should_debug = false;
                    }
                    if let Some(selected_node) = graph_selection
                        .nodes_context
                        .get_selected_nodes()
                        .iter()
                        .rev()
                        .find(|id| **id > 1)
                        && *selected_node > 1
                    {
                        let node_name = graph_indices.node_indices.name(*selected_node).unwrap();
                        graph.nodes.get_mut(node_name).unwrap().should_debug = true;
                        if let InspectorSelection::Node(node_selection) =
                            &mut ctx.global_state.inspector_selection
                        {
                            if &node_selection.node != node_name
                                || node_selection.graph != graph_selection.graph
                            {
                                node_selection.node.clone_from(node_name);
                                node_selection.name_buf.clone_from(node_name);
                                node_selection.graph = graph_selection.graph.clone();
                            }
                        } else if graph_selection.nodes_context.is_node_just_selected() {
                            ctx.global_state.inspector_selection =
                                InspectorSelection::Node(NodeSelection {
                                    graph: graph_selection.graph.clone(),
                                    node: node_name.clone(),
                                    name_buf: node_name.clone(),
                                    selected_pin_id: None,
                                });
                        }
                    }
                    // ----------------------------------------------------------------
                });
            });
        });

        let available_size = ui.available_size();
        let (id, rect) = ui.allocate_space(available_size);

        CustomPopup::new()
            .with_salt(id.with("Graph editor right click popup"))
            .with_sense_rect(rect)
            .with_allow_opening(true)
            .with_save_on_click(Some(()))
            .with_default_size(egui::Vec2::new(500., 300.))
            .show_if_saved(ui, |ui, ()| {
                self.node_creator_popup(ui, world, ctx);
            });
    }

    fn display_name(&self) -> String {
        "Graph Editor".to_string()
    }
}

impl GraphEditorWindow {
    fn node_creator_popup(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut LegacyEditorWindowContext,
    ) {
        let Some(mut types): Option<Vec<_>> = world
            .get_resource::<AppTypeRegistry>()
            .map(|atr| atr.0.read())
            .map(|tr| {
                Self::get_all_nodes_type_info(&tr)
                    .into_iter()
                    .filter(|ty| {
                        let query = &ctx.global_state.node_creation.node_type_search;

                        if query.is_empty() {
                            true
                        } else if query.chars().any(|c| c.is_uppercase()) {
                            ty.path.contains(query)
                        } else {
                            ty.path.to_lowercase().contains(query)
                        }
                    })
                    .collect()
            })
        else {
            return;
        };

        types.sort_unstable_by(|a, b| a.path.cmp(&b.path));

        let original_type_id = ctx.global_state.node_creation.node.inner.type_id();
        let mut new_type_id = original_type_id;

        egui::SidePanel::left("Node type selector")
            .resizable(true)
            .default_width(175.)
            .min_width(150.)
            .show_inside(ui, |ui| {
                ui.text_edit_singleline(&mut ctx.global_state.node_creation.node_type_search);
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        for type_info in &types {
                            let response = ui
                                .selectable_label(false, &type_info.short)
                                .on_hover_text(&type_info.path);
                            if response.clicked() {
                                new_type_id = type_info.id;
                            }
                        }
                    });
            });

        if new_type_id != original_type_id {
            // TODO actual error handling
            if let Some(type_registry) = world
                .get_resource::<AppTypeRegistry>()
                .map(|atr| atr.0.read())
            {
                let _ = (|| {
                    let reflect_default = type_registry
                        .get_type_data::<ReflectDefault>(new_type_id)
                        .ok_or("type doesn't `#[reflect(Default)]`")?;
                    let node_like = type_registry
                        .get_type_data::<ReflectNodeLike>(new_type_id)
                        .ok_or("type doesn't `#[reflect(NodeLike)]`")?;
                    let inner = node_like
                        .get_boxed(reflect_default.default())
                        .map_err(|_| "default-created value is not a `NodeLike`")?;
                    ctx.global_state.node_creation.node.inner = inner;
                    Ok::<_, &str>(())
                })();
            }
        }

        egui::Frame::new().outer_margin(3).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    using_inspector_env(world, |mut env| {
                        let node = &mut ctx.global_state.node_creation.node;

                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            env.ui_for_reflect_with_options(
                                &mut node.name,
                                ui,
                                ui.id().with("Node creator name edit"),
                                &(),
                            );
                        });

                        env.ui_for_reflect_with_options(
                            node.inner.as_partial_reflect_mut(),
                            ui,
                            ui.id().with("Create node reflect"),
                            &(),
                        );
                    });

                    let submit_response = ui
                        .with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                            ui.button("Create node")
                        })
                        .inner;

                    if submit_response.clicked() && ctx.global_state.graph_editor.is_some() {
                        let graph_selection = ctx.global_state.graph_editor.as_ref().unwrap();

                        ctx.editor_actions
                            .push(EditorAction::Graph(GraphAction::CreateNode(CreateNode {
                                graph: graph_selection.graph.clone(),
                                node: ctx.global_state.node_creation.node.clone(),
                            })));
                    }
                });
        });
    }

    fn get_all_nodes_type_info(type_registry: &TypeRegistry) -> Vec<TypeInfo> {
        type_registry
            .iter_with_data::<ReflectNodeLike>()
            .map(|(registration, _)| {
                let path = registration.type_info().type_path().to_string();

                // `bevy_animation_graph::node::f32::Add` ->
                // - `Add`
                // - `f32::Add`
                // - `node::f32::Add`
                // - `bevy_animation_graph::node::f32::Add`
                let mut segments = Vec::new();
                for segment in path.split("::") {
                    segments.push(segment.to_string());
                }

                TypeInfo {
                    id: registration.type_id(),
                    path,
                    short: if let Some(last) = segments.last() {
                        last.clone()
                    } else {
                        debug_assert!(false, "Did not find a short type name for node");
                        String::from("<No type path>")
                    },
                }
            })
            .collect::<Vec<_>>()
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
