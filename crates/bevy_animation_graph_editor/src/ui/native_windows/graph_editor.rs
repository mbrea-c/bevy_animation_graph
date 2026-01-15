use std::any::TypeId;

use bevy::{
    asset::{AssetId, Assets, Handle},
    ecs::reflect::AppTypeRegistry,
    prelude::World,
    reflect::{TypeRegistry, prelude::ReflectDefault},
    utils::default,
};
use bevy_animation_graph::core::{
    animation_graph::{AnimationGraph, NodeId},
    animation_node::{AnimationNode, ReflectNodeLike, dyn_node_like::DynNodeLike},
    context::spec_context::SpecResources,
    state_machine::high_level::StateMachine,
};
use egui_dock::egui;
use uuid::Uuid;

use crate::{
    egui_nodes::lib::{GraphChange as EguiGraphChange, NodesContext},
    graph_show::{GraphIndices, GraphIndicesMap, GraphReprSpec, Pin},
    ui::{
        actions::{
            EditorAction,
            graph::{
                CreateLink, CreateNode, GenerateIndices, GraphAction, MoveInput, MoveNode,
                MoveOutput, RemoveLink, RemoveNode,
            },
        },
        generic_widgets::animation_node::AnimationNodeWidget,
        native_windows::{EditorWindowContext, NativeEditorWindowExtension},
        state_management::global::{
            active_graph::ActiveGraph,
            active_graph_context::ActiveContexts,
            active_graph_node::{ActiveGraphNode, SetActiveGraphNode},
            get_global_state,
            inspector_selection::{InspectorSelection, SetInspectorSelection},
        },
        utils::{self, dummy_node, popup::CustomPopup},
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

#[derive(Default)]
pub struct GraphEditBuffer {
    pub graph: AssetId<AnimationGraph>,
    pub nodes_context: NodesContext,
}

impl NativeEditorWindowExtension for GraphEditorWindow {
    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let Some(active_graph) = get_global_state::<ActiveGraph>(world).cloned() else {
            ui.centered_and_justified(|ui| ui.label("Select a graph to edit!"));
            return;
        };

        let buffer_id = ui.id().with("Graph editor nodes context buffer");
        let buffer = ctx
            .buffers
            .get_mut_or_insert_with(buffer_id, || GraphEditBuffer {
                graph: active_graph.handle.id(),
                ..default()
            });
        let buffer = if buffer.graph != active_graph.handle.id() {
            ctx.buffers.clear::<GraphEditBuffer>(buffer_id);
            ctx.buffers
                .get_mut_or_insert_with(buffer_id, || GraphEditBuffer {
                    graph: active_graph.handle.id(),
                    ..default()
                })
        } else {
            buffer
        };

        let result =
            world.resource_scope::<Assets<AnimationGraph>, _>(|world, mut graph_assets| {
                world.resource_scope::<Assets<StateMachine>, _>(|world, fsm_assets| {
                    world.resource_scope::<GraphIndicesMap, _>(|world, graph_indices_map| {
                        let graph = graph_assets.get(&active_graph.handle)?;
                        let Some(graph_indices) =
                            graph_indices_map.indices.get(&active_graph.handle.id())
                        else {
                            ctx.editor_actions.push(EditorAction::Graph(
                                GraphAction::GenerateIndices(GenerateIndices {
                                    graph: active_graph.handle.id(),
                                }),
                            ));
                            return None;
                        };

                        {
                            let spec_resources = SpecResources {
                                graph_assets: &graph_assets,
                                fsm_assets: &fsm_assets,
                            };

                            let maybe_graph_context = get_global_state::<ActiveContexts>(world)
                                .and_then(|s| s.by_asset.get(&active_graph.handle.id().untyped()))
                                .and_then(|(entity, id)| {
                                    Some((
                                        id,
                                        utils::get_specific_animation_graph_player(world, *entity)?,
                                    ))
                                })
                                .and_then(|(id, p)| Some(id).zip(p.get_context_arena()))
                                .and_then(|(id, ca)| ca.get_context(*id));

                            let nodes = GraphReprSpec::from_graph(
                                graph,
                                graph_indices,
                                spec_resources,
                                maybe_graph_context,
                            );

                            buffer.nodes_context.show(nodes.nodes, nodes.edges, ui);
                            buffer.nodes_context.get_changes().clone()
                        }
                        .into_iter()
                        .map(|c| {
                            convert_graph_change(c, graph_indices, active_graph.handle.clone())
                        })
                        .for_each(|action| ctx.editor_actions.push(EditorAction::Graph(action)));

                        // --- Update selection for node inspector.
                        // --- And enable debug render for latest node selected only
                        // ----------------------------------------------------------------

                        let graph = graph_assets.get_mut(&active_graph.handle).unwrap();
                        for (_, node) in graph.nodes.iter_mut() {
                            node.should_debug = false;
                        }
                        if let Some(selected_node) = buffer
                            .nodes_context
                            .get_selected_nodes()
                            .iter()
                            .rev()
                            .find(|id| **id > 1)
                            && *selected_node > 1
                        {
                            let node_id = graph_indices.node_indices.name(*selected_node).unwrap();
                            graph.nodes.get_mut(&node_id).unwrap().should_debug = true;
                            if let Some(active_node) = get_global_state::<ActiveGraphNode>(world)
                                && let Some(InspectorSelection::ActiveNode) =
                                    get_global_state::<InspectorSelection>(world)
                                && active_node.node == node_id
                                && active_node.handle == active_graph.handle
                            {
                                // pass
                            } else {
                                return Some((
                                    SetActiveGraphNode {
                                        new: ActiveGraphNode {
                                            handle: active_graph.handle.clone(),
                                            node: node_id,
                                            selected_pin: None,
                                        },
                                    },
                                    SetInspectorSelection {
                                        selection: InspectorSelection::ActiveNode,
                                    },
                                ));
                            }
                        }
                        // ----------------------------------------------------------------
                        None
                    })
                })
            });
        if let Some((set_active_graph_node, set_inspector_selection)) = result {
            ctx.trigger(set_active_graph_node);
            ctx.trigger(set_inspector_selection);
        }

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

pub struct NodeCreationBuffer {
    pub node_type_search: String,
    pub node: AnimationNode,
}

impl Default for NodeCreationBuffer {
    fn default() -> Self {
        Self {
            node_type_search: "".into(),
            node: dummy_node(),
        }
    }
}

impl GraphEditorWindow {
    fn node_creator_popup(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut EditorWindowContext,
    ) {
        let buffer_id = ui.id().with("node creator popup");
        let buffer = ctx
            .buffers
            .get_mut_or_default::<NodeCreationBuffer>(buffer_id);

        let Some(mut types): Option<Vec<_>> = world
            .get_resource::<AppTypeRegistry>()
            .map(|atr| atr.0.read())
            .map(|tr| {
                Self::get_all_nodes_type_info(&tr)
                    .into_iter()
                    .filter(|ty| {
                        let query = &buffer.node_type_search;

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

        let original_type_id = buffer.node.inner.type_id();
        let mut new_type_id = original_type_id;

        egui::SidePanel::left("Node type selector")
            .resizable(true)
            .default_width(175.)
            .min_width(150.)
            .show_inside(ui, |ui| {
                ui.text_edit_singleline(&mut buffer.node_type_search);
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
                    buffer.node.inner = DynNodeLike::new_boxed(inner);
                    Ok::<_, &str>(())
                })();
            }
        }

        egui::Frame::new().outer_margin(3).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.add(AnimationNodeWidget::new_salted(
                        &mut buffer.node,
                        world,
                        "create animation node widget",
                    ));

                    let submit_response = ui
                        .with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                            ui.button("Create node")
                        })
                        .inner;

                    if submit_response.clicked()
                        && let Some(active_graph) = get_global_state::<ActiveGraph>(world)
                    {
                        let mut node = buffer.node.clone();
                        node.id = NodeId::from(Uuid::new_v4());
                        ctx.editor_actions
                            .push(EditorAction::Graph(GraphAction::CreateNode(CreateNode {
                                graph: active_graph.handle.clone(),
                                node,
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
