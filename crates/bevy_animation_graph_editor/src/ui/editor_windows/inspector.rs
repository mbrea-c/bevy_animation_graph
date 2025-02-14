use std::{
    any::TypeId,
    hash::{Hash, Hasher},
};

use bevy::{
    asset::Assets,
    ecs::world::CommandQueue,
    log::warn,
    prelude::{AppTypeRegistry, ReflectDefault, World},
    utils::HashMap,
};
use bevy_animation_graph::{
    core::state_machine::high_level::{State, StateMachine, Transition},
    prelude::{AnimationGraph, ReflectEditProxy, ReflectNodeLike},
};
use bevy_inspector_egui::reflect_inspector::{Context, InspectorUi};
use egui_dock::egui;

use crate::{
    graph_update::{Change, FsmChange, FsmPropertiesChange, GlobalChange, GraphChange},
    ui::{
        core::{EditorContext, EditorWindowExtension, FsmSelection, InspectorSelection},
        utils,
    },
};

#[derive(Debug)]
pub struct InspectorWindow;

impl EditorWindowExtension for InspectorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorContext) {
        match ctx.selection.inspector_selection {
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

fn node_inspector(world: &mut World, ui: &mut egui::Ui, ctx: &mut EditorContext) {
    ui.heading("Graph node");

    let mut changes = Vec::new();

    let InspectorSelection::Node(node_selection) = &mut ctx.selection.inspector_selection else {
        return;
    };

    let unsafe_world = world.as_unsafe_world_cell();
    let type_registry = unsafe {
        unsafe_world
            .get_resource::<AppTypeRegistry>()
            .unwrap()
            .0
            .clone()
    };

    let mut graph_assets = unsafe {
        unsafe_world
            .get_resource_mut::<Assets<AnimationGraph>>()
            .unwrap()
    };
    let graph = graph_assets.get_mut(node_selection.graph).unwrap();
    let Some(node) = graph.nodes.get_mut(&node_selection.node) else {
        ctx.selection.inspector_selection = InspectorSelection::Nothing;
        return;
    };

    let response = ui.text_edit_singleline(&mut node_selection.name_buf);
    if response.lost_focus() {
        changes.push(GraphChange {
            change: Change::NodeRenamed(
                node_selection.node.clone(),
                node_selection.name_buf.clone(),
            ),
            graph: node_selection.graph,
        });
    }

    let type_registry = type_registry.read();
    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(unsafe { unsafe_world.world_mut() }.into()),
        queue: Some(&mut queue),
    };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

    // TODO: Make node update into a GraphChange
    // (eventually we want all graph mutations to go through GraphChange, this
    // will enable easier undo/redo support)
    let changed = if let Some(edit_proxy) =
        type_registry.get_type_data::<ReflectEditProxy>(node.inner.type_id())
    {
        let mut proxy = (edit_proxy.to_proxy)(node.inner.as_ref());
        let changed = env.ui_for_reflect(proxy.as_partial_reflect_mut(), ui);
        if changed {
            let inner = (edit_proxy.from_proxy)(proxy.as_ref());
            node.inner = inner;
        }
        changed
    } else {
        let changed = env.ui_for_reflect(node.inner.as_partial_reflect_mut(), ui);
        changed
    };

    if changed {
        changes.push(GraphChange {
            change: Change::GraphValidate,
            graph: node_selection.graph,
        });
    }

    ctx.graph_changes.extend(changes);

    queue.apply(world);
}

fn state_inspector(world: &mut World, ui: &mut egui::Ui, ctx: &mut EditorContext) {
    ui.heading("FSM State");

    let mut changes = Vec::new();

    let Some(_fsm_selection) = &mut ctx.selection.fsm_editor else {
        return;
    };

    let InspectorSelection::FsmState(state_selection) = &mut ctx.selection.inspector_selection
    else {
        return;
    };

    let unsafe_world = world.as_unsafe_world_cell();
    let type_registry = unsafe {
        unsafe_world
            .get_resource::<AppTypeRegistry>()
            .unwrap()
            .0
            .clone()
    };
    let mut fsm_assets = unsafe {
        unsafe_world
            .get_resource_mut::<Assets<StateMachine>>()
            .unwrap()
    };
    let fsm = fsm_assets.get_mut(state_selection.fsm).unwrap();
    let Some(state) = fsm.states.get_mut(&state_selection.state) else {
        ctx.selection.inspector_selection = InspectorSelection::Nothing;
        return;
    };

    let type_registry = type_registry.read();
    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(unsafe { unsafe_world.world_mut() }.into()),
        queue: Some(&mut queue),
    };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

    let mut copy = state.clone();

    let changed = env.ui_for_reflect(&mut copy, ui);

    if changed {
        changes.push(GlobalChange::FsmChange {
            asset_id: state_selection.fsm,
            change: FsmChange::StateChanged(state_selection.state.clone(), copy),
        });
    }

    ctx.global_changes.extend(changes);

    queue.apply(world);
}

fn transition_inspector(world: &mut World, ui: &mut egui::Ui, ctx: &mut EditorContext) {
    ui.heading("FSM Transition");
    let mut changes = Vec::new();

    let Some(fsm_selection) = &mut ctx.selection.fsm_editor else {
        return;
    };

    let InspectorSelection::FsmTransition(transition_selection) =
        &mut ctx.selection.inspector_selection
    else {
        return;
    };

    let unsafe_world = world.as_unsafe_world_cell();
    let type_registry = unsafe {
        unsafe_world
            .get_resource::<AppTypeRegistry>()
            .unwrap()
            .0
            .clone()
    };
    let mut fsm_assets = unsafe {
        unsafe_world
            .get_resource_mut::<Assets<StateMachine>>()
            .unwrap()
    };
    let fsm = fsm_assets.get_mut(fsm_selection.fsm).unwrap();
    let Some(transition) = fsm.transitions.get_mut(&transition_selection.state) else {
        ctx.selection.inspector_selection = InspectorSelection::Nothing;
        return;
    };

    let type_registry = type_registry.read();
    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(unsafe { unsafe_world.world_mut() }.into()),
        queue: Some(&mut queue),
    };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

    let mut copy = transition.clone();

    let changed = env.ui_for_reflect(&mut copy, ui);

    if changed {
        println!("Should push a change now");
        changes.push(GlobalChange::FsmChange {
            asset_id: transition_selection.fsm,
            change: FsmChange::TransitionChanged(transition_selection.state.clone(), copy),
        });
    }

    ctx.global_changes.extend(changes);

    queue.apply(world);
}

fn graph_inspector(world: &mut World, ui: &mut egui::Ui, ctx: &mut EditorContext) {
    ui.heading("Animation graph");

    utils::select_graph_context(world, ui, ctx.selection);

    ui.collapsing("Create node", |ui| node_creator(world, ui, ctx));

    let mut changes = Vec::new();

    let Some(graph_selection) = &mut ctx.selection.graph_editor else {
        return;
    };

    let unsafe_world = world.as_unsafe_world_cell();
    let type_registry = unsafe {
        unsafe_world
            .get_resource::<AppTypeRegistry>()
            .unwrap()
            .0
            .clone()
    };
    let mut graph_assets = unsafe {
        unsafe_world
            .get_resource_mut::<Assets<AnimationGraph>>()
            .unwrap()
    };
    let graph = graph_assets.get_mut(graph_selection.graph).unwrap();

    let type_registry = type_registry.read();
    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(unsafe { unsafe_world.world_mut() }.into()),
        queue: Some(&mut queue),
    };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

    let changed =
        env.ui_for_reflect_with_options(graph, ui, egui::Id::new(graph_selection.graph), &());

    if changed {
        changes.push(GraphChange {
            change: Change::GraphValidate,
            graph: graph_selection.graph,
        });
    }

    ctx.graph_changes.extend(changes);

    queue.apply(world);
}

fn fsm_inspector(world: &mut World, ui: &mut egui::Ui, ctx: &mut EditorContext) {
    ui.heading("State machine");
    let mut changes = Vec::new();

    utils::select_graph_context_fsm(world, ui, ctx.selection);

    let Some(fsm_selection) = &mut ctx.selection.fsm_editor else {
        return;
    };

    let unsafe_world = world.as_unsafe_world_cell();
    let type_registry = unsafe {
        unsafe_world
            .get_resource::<AppTypeRegistry>()
            .unwrap()
            .0
            .clone()
    };
    let fsm_assets = unsafe { unsafe_world.get_resource::<Assets<StateMachine>>().unwrap() };
    let fsm = fsm_assets.get(fsm_selection.fsm).unwrap();

    let type_registry = type_registry.read();
    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(unsafe { unsafe_world.world_mut() }.into()),
        queue: Some(&mut queue),
    };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

    let mut properties = FsmPropertiesChange::from(fsm);

    let changed =
        env.ui_for_reflect_with_options(&mut properties, ui, egui::Id::new(fsm_selection.fsm), &());
    if changed {
        changes.push(GlobalChange::FsmChange {
            asset_id: fsm_selection.fsm,
            change: FsmChange::PropertiesChanged(properties),
        });
    }

    if let Some(new_state) = add_state_ui(ui, fsm_selection, &mut env) {
        changes.push(GlobalChange::FsmChange {
            asset_id: fsm_selection.fsm,
            change: FsmChange::StateAdded(new_state),
        })
    }

    if let Some(transition) = add_transition_ui(ui, fsm_selection, &mut env) {
        changes.push(GlobalChange::FsmChange {
            asset_id: fsm_selection.fsm,
            change: FsmChange::TransitionAdded(transition),
        })
    }

    ctx.global_changes.extend(changes);
    queue.apply(world);
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

fn node_creator(world: &mut World, ui: &mut egui::Ui, ctx: &mut EditorContext) {
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

    let original_type_id = ctx.selection.node_creation.node.inner.type_id();
    let mut type_id = original_type_id;
    egui::Grid::new("node creator fields")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut ctx.selection.node_creation.node.name);
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
                    ctx.selection
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
            ctx.selection.node_creation.node.inner = inner;
            Ok::<_, &str>(())
        })();

        if let Err(err) = result {
            warn!("Failed to start creating node of type {type_id:?}: {err}");
        }
    }

    let submit_response = ui.button("Create node");

    if submit_response.clicked() && ctx.selection.graph_editor.is_some() {
        let graph_selection = ctx.selection.graph_editor.as_ref().unwrap();
        ctx.graph_changes.push(GraphChange {
            change: Change::NodeCreated(ctx.selection.node_creation.node.clone()),
            graph: graph_selection.graph,
        });
    }

    queue.apply(world);
}
