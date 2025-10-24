use std::{
    any::TypeId,
    hash::{Hash, Hasher},
};

use bevy::{
    asset::Assets,
    ecs::world::CommandQueue,
    log::warn,
    platform::collections::HashMap,
    prelude::{AppTypeRegistry, ReflectDefault, World},
};
use bevy_animation_graph::{
    core::state_machine::high_level::{State, StateMachine, Transition},
    prelude::{AnimationGraph, ReflectEditProxy, ReflectNodeLike},
};
use bevy_inspector_egui::reflect_inspector::{Context, InspectorUi};
use egui_dock::egui;

use crate::ui::{
    actions::{
        EditorAction,
        fsm::{
            CreateState, CreateTransition, FsmAction, FsmProperties, UpdateProperties, UpdateState,
            UpdateTransition,
        },
        graph::{
            CreateNode, EditNode, GraphAction, RenameNode, UpdateInputData, UpdateInputTimes,
            UpdateOutputData, UpdateOutputTime,
        },
    },
    core::{EditorWindowExtension, FsmSelection, InspectorSelection, LegacyEditorWindowContext},
    egui_inspector_impls::OrderedMap,
    utils::{self, using_inspector_env},
};

#[derive(Debug)]
pub struct InspectorWindow;

TODO

impl EditorWindowExtension for InspectorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut LegacyEditorWindowContext) {
        match ctx.global_state.inspector_selection {
            InspectorSelection::FsmTransition(_) => transition_inspector(world, ui, ctx),
            InspectorSelection::FsmState(_) => state_inspector(world, ui, ctx),
            InspectorSelection::Node(_) => node_inspector(world, ui, ctx),
            InspectorSelection::Graph => graph_inspector(world, ui, ctx),
            InspectorSelection::Fsm => fsm_inspector(world, ui, ctx),
            InspectorSelection::Nothing => {}
        }
    }

    fn display_name(&self) -> String {
        "Inspector".to_string()
    }
}

fn node_inspector(world: &mut World, ui: &mut egui::Ui, ctx: &mut LegacyEditorWindowContext) {
    ui.heading("Graph node");

    world.resource_scope::<Assets<AnimationGraph>, _>(|world, graph_assets| {
        let InspectorSelection::Node(node_selection) = &mut ctx.global_state.inspector_selection
        else {
            return;
        };

        let graph = graph_assets.get(&node_selection.graph).unwrap();
        let Some(node) = graph.nodes.get(&node_selection.node) else {
            ctx.global_state.inspector_selection = InspectorSelection::Nothing;
            return;
        };

        let response = ui.text_edit_singleline(&mut node_selection.name_buf);
        if response.lost_focus() {
            ctx.editor_actions
                .push(EditorAction::Graph(GraphAction::RenameNode(RenameNode {
                    graph: node_selection.graph.clone(),
                    node: node_selection.node.clone(),
                    new_name: node_selection.name_buf.clone(),
                })));
        }

        let changed = using_inspector_env(world, |mut env| {
            if let Some(edit_proxy) = env
                .type_registry
                .get_type_data::<ReflectEditProxy>(node.inner.type_id())
            {
                let mut proxy = (edit_proxy.to_proxy)(node.inner.as_ref());
                let changed = env.ui_for_reflect_with_options(
                    proxy.as_partial_reflect_mut(),
                    ui,
                    ui.id(),
                    &(),
                );
                changed.then(|| (edit_proxy.from_proxy)(proxy.as_ref()))
            } else {
                let mut current_inner = node.inner.clone();
                let changed = env.ui_for_reflect(current_inner.as_partial_reflect_mut(), ui);
                changed.then_some(current_inner)
            }
        });

        if let Some(changed) = changed {
            ctx.editor_actions
                .push(EditorAction::Graph(GraphAction::EditNode(EditNode {
                    graph: node_selection.graph.clone(),
                    node: node.name.clone(),
                    new_inner: changed,
                })));
        }
    });
}

fn state_inspector(world: &mut World, ui: &mut egui::Ui, ctx: &mut LegacyEditorWindowContext) {
    ui.heading("FSM State");

    world.resource_scope::<Assets<StateMachine>, _>(|world, fsm_assets| {
        let InspectorSelection::FsmState(state_selection) =
            &mut ctx.global_state.inspector_selection
        else {
            return;
        };
        let fsm = fsm_assets.get(&state_selection.fsm).unwrap();

        let Some(state) = fsm.states.get(&state_selection.state) else {
            ctx.global_state.inspector_selection = InspectorSelection::Nothing;
            return;
        };

        using_inspector_env(world, |mut env| {
            let mut copy = state.clone();
            let changed = env.ui_for_reflect(&mut copy, ui);
            if changed {
                ctx.editor_actions
                    .push(EditorAction::Fsm(FsmAction::UpdateState(UpdateState {
                        fsm: state_selection.fsm.clone(),
                        state_id: state_selection.state.clone(),
                        new_state: copy,
                    })));
            }
        });
    });
}

fn transition_inspector(world: &mut World, ui: &mut egui::Ui, ctx: &mut LegacyEditorWindowContext) {
    ui.heading("FSM Transition");

    world.resource_scope::<Assets<StateMachine>, _>(|world, fsm_assets| {
        let Some(fsm_selection) = &mut ctx.global_state.fsm_editor else {
            return;
        };
        let InspectorSelection::FsmTransition(transition_selection) =
            &mut ctx.global_state.inspector_selection
        else {
            return;
        };
        let fsm = fsm_assets.get(&fsm_selection.fsm).unwrap();
        let Some(transition) = fsm.transitions.get(&transition_selection.state) else {
            ctx.global_state.inspector_selection = InspectorSelection::Nothing;
            return;
        };

        using_inspector_env(world, |mut env| {
            let mut copy = transition.clone();
            let changed = env.ui_for_reflect(&mut copy, ui);
            if changed {
                ctx.editor_actions
                    .push(EditorAction::Fsm(FsmAction::UpdateTransition(
                        UpdateTransition {
                            fsm: transition_selection.fsm.clone(),
                            transition_id: transition_selection.state.clone(),
                            new_transition: copy,
                        },
                    )));
            }
        });
    });
}

fn graph_inspector(world: &mut World, ui: &mut egui::Ui, ctx: &mut LegacyEditorWindowContext) {
    ui.heading("Animation graph");

    utils::select_graph_context(world, ui, ctx.global_state);

    ui.collapsing("Create node", |ui| node_creator(world, ui, ctx));

    let Some(graph_selection) = &mut ctx.global_state.graph_editor else {
        return;
    };

    world.resource_scope::<Assets<AnimationGraph>, _>(|world, graph_assets| {
        let graph = graph_assets.get(&graph_selection.graph).unwrap();

        using_inspector_env(world, |mut env| {
            let mut input_data_changed = false;
            let mut output_data_changed = false;
            let mut input_times_changed = false;
            let mut output_time_changed = false;

            let mut input_data = OrderedMap {
                order: graph.extra.input_param_order.clone(),
                values: graph.default_parameters.clone(),
            };

            ui.collapsing("Default input data", |ui| {
                input_data_changed = env.ui_for_reflect_with_options(
                    &mut input_data,
                    ui,
                    ui.id().with("default input data"),
                    &(),
                );
            });

            let mut output_data = OrderedMap {
                order: graph.extra.output_data_order.clone(),
                values: graph.output_parameters.clone(),
            };
            ui.collapsing("Output data", |ui| {
                output_data_changed = env.ui_for_reflect_with_options(
                    &mut output_data,
                    ui,
                    ui.id().with("output data"),
                    &(),
                );
            });

            let mut input_times = OrderedMap {
                order: graph.extra.input_time_order.clone(),
                values: graph.input_times.clone(),
            };
            ui.collapsing("Input times", |ui| {
                input_times_changed = env.ui_for_reflect_with_options(
                    &mut input_times,
                    ui,
                    ui.id().with("input times"),
                    &(),
                );
            });

            let mut output_time = graph.output_time;

            ui.collapsing("Output time", |ui| {
                output_time_changed = env.ui_for_reflect_with_options(
                    &mut output_time,
                    ui,
                    ui.id().with("output time"),
                    &(),
                );
            });

            if input_data_changed {
                ctx.editor_actions
                    .push(EditorAction::Graph(GraphAction::UpdateInputData(
                        UpdateInputData {
                            graph: graph_selection.graph.clone(),
                            input_data,
                        },
                    )));
            }

            if output_data_changed {
                ctx.editor_actions
                    .push(EditorAction::Graph(GraphAction::UpdateOutputData(
                        UpdateOutputData {
                            graph: graph_selection.graph.clone(),
                            output_data,
                        },
                    )));
            }

            if input_times_changed {
                ctx.editor_actions
                    .push(EditorAction::Graph(GraphAction::UpdateInputTimes(
                        UpdateInputTimes {
                            graph: graph_selection.graph.clone(),
                            input_times,
                        },
                    )));
            }

            if output_time_changed {
                ctx.editor_actions
                    .push(EditorAction::Graph(GraphAction::UpdateOutputTime(
                        UpdateOutputTime {
                            graph: graph_selection.graph.clone(),
                            output_time,
                        },
                    )));
            }
        });
    });
}

fn fsm_inspector(world: &mut World, ui: &mut egui::Ui, ctx: &mut LegacyEditorWindowContext) {
    ui.heading("State machine");

    utils::select_graph_context_fsm(world, ui, ctx.global_state);

    let Some(fsm_selection) = &mut ctx.global_state.fsm_editor else {
        return;
    };

    world.resource_scope::<Assets<StateMachine>, _>(|world, fsm_assets| {
        let fsm = fsm_assets.get(&fsm_selection.fsm).unwrap();

        using_inspector_env(world, |mut env| {
            let mut new_properties = FsmProperties::from(fsm);

            let changed = env.ui_for_reflect_with_options(
                &mut new_properties,
                ui,
                ui.id().with("fsm properties inspector"),
                &(),
            );
            if changed {
                ctx.editor_actions
                    .push(EditorAction::Fsm(FsmAction::UpdateProperties(
                        UpdateProperties {
                            fsm: fsm_selection.fsm.clone(),
                            new_properties,
                        },
                    )));
            }

            if let Some(state) = add_state_ui(ui, fsm_selection, &mut env) {
                ctx.editor_actions
                    .push(EditorAction::Fsm(FsmAction::CreateState(CreateState {
                        fsm: fsm_selection.fsm.clone(),
                        state,
                    })));
            }

            if let Some(transition) = add_transition_ui(ui, fsm_selection, &mut env) {
                ctx.editor_actions
                    .push(EditorAction::Fsm(FsmAction::CreateTransition(
                        CreateTransition {
                            fsm: fsm_selection.fsm.clone(),
                            transition,
                        },
                    )));
            }
        });
    });
}

fn add_transition_ui(
    ui: &mut egui::Ui,
    fsm_selection: &mut FsmSelection,
    env: &mut InspectorUi,
) -> Option<Transition> {
    ui.separator();
    ui.label("Transition creation");
    env.ui_for_reflect_with_options(
        &mut fsm_selection.transition_creation,
        ui,
        egui::Id::new("Transition creation"),
        &(),
    );
    if ui.button("Create transition").clicked() {
        Some(fsm_selection.transition_creation.clone())
    } else {
        None
    }
}

fn add_state_ui(
    ui: &mut egui::Ui,
    fsm_selection: &mut FsmSelection,
    env: &mut InspectorUi,
) -> Option<State> {
    ui.separator();
    ui.label("State creation");
    env.ui_for_reflect_with_options(
        &mut fsm_selection.state_creation,
        ui,
        egui::Id::new("State creation"),
        &(),
    );
    if ui.button("Create state").clicked() {
        Some(fsm_selection.state_creation.clone())
    } else {
        None
    }
}

fn node_creator(world: &mut World, ui: &mut egui::Ui, ctx: &mut LegacyEditorWindowContext) {
    let unsafe_world = world.as_unsafe_world_cell();
    let type_registry = unsafe {
        unsafe_world
            .get_resource::<AppTypeRegistry>()
            .unwrap()
            .0
            .clone()
    };
    let type_registry = type_registry.read();

    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(unsafe { unsafe_world.world_mut() }.into()),
        queue: Some(&mut queue),
    };

    let original_type_id = ctx.global_state.node_creation.node.inner.type_id();
    let mut type_id = original_type_id;
    egui::Grid::new("node creator fields")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut ctx.global_state.node_creation.node.name);
            ui.end_row();

            ui.label("Type");
            {
                struct TypeInfo<'a> {
                    id: TypeId,
                    path: &'a str,
                    segments: Vec<&'a str>,
                    short: String,
                }

                let mut segments_found = HashMap::<Vec<&str>, usize>::new();
                let mut types = type_registry
                    .iter_with_data::<ReflectNodeLike>()
                    .map(|(registration, _)| {
                        let path = registration.type_info().type_path();

                        // `bevy_animation_graph::node::f32::Add` ->
                        // - `Add`
                        // - `f32::Add`
                        // - `node::f32::Add`
                        // - `bevy_animation_graph::node::f32::Add`
                        let mut segments = Vec::new();
                        for segment in path.rsplit("::") {
                            segments.insert(0, segment);
                            *segments_found.entry(segments.clone()).or_default() += 1;
                        }

                        TypeInfo {
                            id: registration.type_id(),
                            path,
                            segments,
                            short: String::new(),
                        }
                    })
                    .collect::<Vec<_>>();
                for type_info in &mut types {
                    let mut segments = Vec::new();
                    for segment in type_info.segments.iter().rev() {
                        segments.insert(0, *segment);
                        if segments_found.get(&segments).copied().unwrap_or_default() <= 1 {
                            // we've found the shortest unique path starting from the right
                            type_info.short = segments.join("::");
                            break;
                        }
                    }

                    debug_assert!(
                        !type_info.short.is_empty(),
                        "should have found a short type path for `{}`",
                        type_info.path
                    );
                }
                let longest_short_name = types
                    .iter()
                    .map(|type_info| type_info.short.len())
                    .max()
                    .unwrap_or_default();
                types.sort_unstable_by(|a, b| a.path.cmp(b.path));

                let selected_text = types
                    .iter()
                    .find(|type_info| type_info.id == type_id)
                    .map(|type_info| type_info.short.clone())
                    .unwrap_or_else(|| "(?)".into());
                egui::ComboBox::from_id_salt("node creator type")
                    .selected_text(egui::RichText::new(selected_text).monospace())
                    .show_ui(ui, |ui| {
                        for node_type in types {
                            let padding = " ".repeat(longest_short_name - node_type.short.len());
                            let name = format!("{}{padding}  {}", node_type.short, node_type.path);
                            let name = egui::RichText::new(name).monospace();
                            ui.selectable_value(&mut type_id, node_type.id, name);
                        }
                    });
            }
            ui.end_row();

            ui.label("Node");
            {
                let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                "Create node".hash(&mut hasher);
                let node_creator_id = egui::Id::new(Hasher::finish(&hasher));
                env.ui_for_reflect_with_options(
                    ctx.global_state
                        .node_creation
                        .node
                        .inner
                        .as_partial_reflect_mut(),
                    ui,
                    node_creator_id,
                    &(),
                );
            }
            ui.end_row();
        });

    if type_id != original_type_id {
        // TODO actual error handling
        let result = (|| {
            let reflect_default = type_registry
                .get_type_data::<ReflectDefault>(type_id)
                .ok_or("type doesn't `#[reflect(Default)]`")?;
            let node_like = type_registry
                .get_type_data::<ReflectNodeLike>(type_id)
                .ok_or("type doesn't `#[reflect(NodeLike)]`")?;
            let inner = node_like
                .get_boxed(reflect_default.default())
                .map_err(|_| "default-created value is not a `NodeLike`")?;
            ctx.global_state.node_creation.node.inner = inner;
            Ok::<_, &str>(())
        })();

        if let Err(err) = result {
            warn!("Failed to start creating node of type {type_id:?}: {err}");
        }
    }

    let submit_response = ui.button("Create node");

    if submit_response.clicked() && ctx.global_state.graph_editor.is_some() {
        let graph_selection = ctx.global_state.graph_editor.as_ref().unwrap();

        ctx.editor_actions
            .push(EditorAction::Graph(GraphAction::CreateNode(CreateNode {
                graph: graph_selection.graph.clone(),
                node: ctx.global_state.node_creation.node.clone(),
            })));
    }

    queue.apply(world);
}
