use bevy::{
    asset::Assets,
    ecs::entity::Entity,
    prelude::{AppTypeRegistry, World},
};
use bevy_animation_graph::{
    core::state_machine::high_level::{State, StateMachine, Transition},
    prelude::{AnimationGraph, AnimationNode, GraphContext, GraphContextId},
};
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui_dock::egui;

use crate::ui::{
    actions::{
        EditorAction,
        fsm::{
            CreateState, CreateTransition, FsmAction, FsmProperties, UpdateProperties, UpdateState,
            UpdateTransition,
        },
        graph::{
            EditNode, GraphAction, RenameNode, UpdateInputData, UpdateInputTimes, UpdateOutputData,
            UpdateOutputTime,
        },
    },
    egui_inspector_impls::OrderedMap,
    global_state::{
        active_fsm::ActiveFsm,
        active_fsm_state::ActiveFsmState,
        active_fsm_transition::ActiveFsmTransition,
        active_graph::ActiveGraph,
        active_graph_context::{ActiveContexts, SetActiveContext},
        active_graph_node::ActiveGraphNode,
        get_global_state,
        inspector_selection::InspectorSelection,
    },
    native_windows::{EditorWindowContext, NativeEditorWindowExtension},
    node_editors::{ReflectEditable, reflect_editor::ReflectNodeEditor},
    utils::{self, using_inspector_env, with_assets_all},
};

#[derive(Debug)]
pub struct InspectorWindow;

impl NativeEditorWindowExtension for InspectorWindow {
    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let inspector_selection = get_global_state::<InspectorSelection>(world)
            .cloned()
            .unwrap_or_default();

        ui.push_id(&inspector_selection, |ui| match inspector_selection {
            InspectorSelection::ActiveFsm => {
                fsm_inspector(world, ui, ctx);
            }
            InspectorSelection::ActiveFsmTransition => {
                transition_inspector(world, ui, ctx);
            }
            InspectorSelection::ActiveFsmState => {
                state_inspector(world, ui, ctx);
            }
            InspectorSelection::ActiveGraph => {
                graph_inspector(world, ui, ctx);
            }
            InspectorSelection::ActiveNode => {
                node_inspector(world, ui, ctx);
            }
            InspectorSelection::Nothing => {}
        });
    }

    fn display_name(&self) -> String {
        "Inspector".to_string()
    }
}

fn node_inspector(
    world: &mut World,
    ui: &mut egui::Ui,
    ctx: &mut EditorWindowContext,
) -> Option<()> {
    ui.heading("Graph node");

    let active_node = get_global_state::<ActiveGraphNode>(world).cloned()?;

    with_assets_all(
        world,
        [active_node.handle.id()],
        |world: &mut World, [graph]| -> Option<_> {
            let node = graph.nodes.get(&active_node.node)?;

            let node_buffer_id = ui.id().with("graph node buffer");

            let node_buffer = ctx
                .buffers
                .get_mut_or_insert_with(node_buffer_id, || node.clone());

            let response = ui.text_edit_singleline(&mut node_buffer.name);
            let mut clear = false;

            if response.lost_focus() {
                ctx.editor_actions
                    .push(EditorAction::Graph(GraphAction::RenameNode(RenameNode {
                        graph: active_node.handle.clone(),
                        node: active_node.node.clone(),
                        new_name: node_buffer.name.clone(),
                    })));
                clear = true;
            }

            let editor = if let Some(editable) = world
                .resource::<AppTypeRegistry>()
                .0
                .clone()
                .read()
                .get_type_data::<ReflectEditable>(node.inner.type_id())
            {
                (editable.get_editor)(node.inner.as_ref())
            } else {
                Box::new(ReflectNodeEditor)
            };

            let inner_edit_response = editor.show_dyn(ui, world, node_buffer.inner.as_mut());

            if inner_edit_response.changed() {
                ctx.editor_actions
                    .push(EditorAction::Graph(GraphAction::EditNode(EditNode {
                        graph: active_node.handle.clone(),
                        node: node.name.clone(),
                        new_inner: node_buffer.inner.clone(),
                    })));
                clear = true;
            }

            if clear {
                ctx.buffers.clear::<AnimationNode>(node_buffer_id);
            }

            Some(())
        },
    )
    .flatten()
}

fn state_inspector(
    world: &mut World,
    ui: &mut egui::Ui,
    ctx: &mut EditorWindowContext,
) -> Option<()> {
    ui.heading("FSM State");

    let active_state = get_global_state::<ActiveFsmState>(world)?.clone();

    with_assets_all(world, [active_state.handle.id()], |world, [fsm]| {
        let state = fsm.states.get(&active_state.state)?;

        using_inspector_env(world, |mut env| {
            let mut copy = state.clone();
            let changed = env.ui_for_reflect(&mut copy, ui);
            if changed {
                ctx.editor_actions
                    .push(EditorAction::Fsm(FsmAction::UpdateState(UpdateState {
                        fsm: active_state.handle.clone(),
                        state_id: active_state.state.clone(),
                        new_state: copy,
                    })));
            }
        });

        Some(())
    })
    .flatten()
}

fn transition_inspector(
    world: &mut World,
    ui: &mut egui::Ui,
    ctx: &mut EditorWindowContext,
) -> Option<()> {
    ui.heading("FSM Transition");

    let active_transition = get_global_state::<ActiveFsmTransition>(world)?.clone();

    with_assets_all(world, [active_transition.handle.id()], |world, [fsm]| {
        let transition = fsm.transitions.get(&active_transition.transition)?;

        using_inspector_env(world, |mut env| {
            let mut copy = transition.clone();
            let changed = env.ui_for_reflect(&mut copy, ui);
            if changed {
                ctx.editor_actions
                    .push(EditorAction::Fsm(FsmAction::UpdateTransition(
                        UpdateTransition {
                            fsm: active_transition.handle.clone(),
                            transition_id: active_transition.transition.clone(),
                            new_transition: copy,
                        },
                    )));
            }
        });
        Some(())
    })
    .flatten()
}

fn graph_inspector(
    world: &mut World,
    ui: &mut egui::Ui,
    ctx: &mut EditorWindowContext,
) -> Option<()> {
    ui.heading("Animation graph");

    select_graph_context(world, ui, ctx);

    let active_graph = get_global_state::<ActiveGraph>(world)?.clone();

    world.resource_scope::<Assets<AnimationGraph>, _>(|world, graph_assets| {
        let graph = graph_assets.get(&active_graph.handle)?;

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
                            graph: active_graph.handle.clone(),
                            input_data,
                        },
                    )));
            }

            if output_data_changed {
                ctx.editor_actions
                    .push(EditorAction::Graph(GraphAction::UpdateOutputData(
                        UpdateOutputData {
                            graph: active_graph.handle.clone(),
                            output_data,
                        },
                    )));
            }

            if input_times_changed {
                ctx.editor_actions
                    .push(EditorAction::Graph(GraphAction::UpdateInputTimes(
                        UpdateInputTimes {
                            graph: active_graph.handle.clone(),
                            input_times,
                        },
                    )));
            }

            if output_time_changed {
                ctx.editor_actions
                    .push(EditorAction::Graph(GraphAction::UpdateOutputTime(
                        UpdateOutputTime {
                            graph: active_graph.handle.clone(),
                            output_time,
                        },
                    )));
            }
        });

        Some(())
    })
}

fn fsm_inspector(
    world: &mut World,
    ui: &mut egui::Ui,
    ctx: &mut EditorWindowContext,
) -> Option<()> {
    ui.heading("State machine");

    select_graph_context_fsm(world, ui, ctx);

    let active_fsm = get_global_state::<ActiveFsm>(world)?.clone();

    world.resource_scope::<Assets<StateMachine>, _>(|world, fsm_assets| {
        let fsm = fsm_assets.get(&active_fsm.handle)?;

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
                            fsm: active_fsm.handle.clone(),
                            new_properties,
                        },
                    )));
            }

            if let Some(state) = add_state_ui(ui, &mut env, ctx) {
                ctx.editor_actions
                    .push(EditorAction::Fsm(FsmAction::CreateState(CreateState {
                        fsm: active_fsm.handle.clone(),
                        state,
                    })));
            }

            if let Some(transition) = add_transition_ui(ui, &mut env, ctx) {
                ctx.editor_actions
                    .push(EditorAction::Fsm(FsmAction::CreateTransition(
                        CreateTransition {
                            fsm: active_fsm.handle.clone(),
                            transition,
                        },
                    )));
            }
        });
        Some(())
    })
}

fn add_transition_ui(
    ui: &mut egui::Ui,
    env: &mut InspectorUi,
    ctx: &mut EditorWindowContext,
) -> Option<Transition> {
    ui.push_id("fsm add transition", |ui| {
        ui.separator();
        ui.label("Transition creation");
        let buffer = ctx
            .buffers
            .get_mut_or_insert_with(ui.id(), || Transition::default());
        env.ui_for_reflect_with_options(buffer, ui, egui::Id::new("Transition creation"), &());
        if ui.button("Create transition").clicked() {
            Some(buffer.clone())
        } else {
            None
        }
    })
    .inner
}

fn add_state_ui(
    ui: &mut egui::Ui,
    env: &mut InspectorUi,
    ctx: &mut EditorWindowContext,
) -> Option<State> {
    ui.push_id("fsm add state", |ui| {
        ui.separator();
        ui.label("State creation");
        let buffer = ctx
            .buffers
            .get_mut_or_insert_with(ui.id(), || State::default());
        env.ui_for_reflect_with_options(buffer, ui, egui::Id::new("State creation"), &());
        if ui.button("Create state").clicked() {
            Some(buffer.clone())
        } else {
            None
        }
    })
    .inner
}

fn select_graph_context(
    world: &mut World,
    ui: &mut egui::Ui,
    ctx: &mut EditorWindowContext,
) -> Option<()> {
    let active_graph = get_global_state::<ActiveGraph>(world)?;
    let active_contexts = get_global_state::<ActiveContexts>(world)?;

    let available_contexts =
        list_graph_contexts(world, |ctx| ctx.get_graph_id() == active_graph.handle.id());

    let mut selected = active_contexts
        .by_asset
        .get(&active_graph.handle.id().untyped())
        .copied();

    let response = egui::ComboBox::from_label("Active context")
        .selected_text(format!("{selected:?}"))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut selected,
                None,
                format!("{:?}", None::<(Entity, GraphContextId)>),
            );
            for (entity, id) in available_contexts {
                ui.selectable_value(
                    &mut selected,
                    Some((entity, id)),
                    format!("{:?} - {:?}", entity, id),
                );
            }
        });

    if let Some((selected_entity, selected_id)) = selected
        && response.response.changed()
    {
        ctx.trigger(SetActiveContext {
            asset_id: active_graph.handle.id().untyped(),
            entity: selected_entity,
            id: selected_id,
        });
    }

    Some(())
}

fn select_graph_context_fsm(
    world: &mut World,
    ui: &mut egui::Ui,
    ctx: &mut EditorWindowContext,
) -> Option<()> {
    let active_fsm = get_global_state::<ActiveFsm>(world)?;
    let active_contexts = get_global_state::<ActiveContexts>(world)?;
    let graph_assets = world.resource::<Assets<AnimationGraph>>();

    let available_contexts = list_graph_contexts(world, |ctx| {
        let graph_id = ctx.get_graph_id();
        let Some(graph) = graph_assets.get(graph_id) else {
            return false;
        };
        graph
            .contains_state_machine(active_fsm.handle.id())
            .is_some()
    });

    let mut selected = active_contexts
        .by_asset
        .get(&active_fsm.handle.id().untyped())
        .copied();

    let response = egui::ComboBox::from_label("Active context")
        .selected_text(format!("{selected:?}"))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut selected,
                None,
                format!("{:?}", None::<(Entity, GraphContextId)>),
            );
            for (entity, id) in available_contexts {
                ui.selectable_value(
                    &mut selected,
                    Some((entity, id)),
                    format!("{:?} - {:?}", entity, id),
                );
            }
        });

    if let Some((selected_entity, selected_id)) = selected
        && response.response.changed()
    {
        ctx.trigger(SetActiveContext {
            asset_id: active_fsm.handle.id().untyped(),
            entity: selected_entity,
            id: selected_id,
        });
    }

    Some(())
}

fn list_graph_contexts(
    world: &World,
    filter: impl Fn(&GraphContext) -> bool,
) -> Vec<(Entity, GraphContextId)> {
    let players = utils::iter_animation_graph_players(world);
    players
        .iter()
        .filter_map(|(entity, player)| Some((entity, player.get_context_arena()?)))
        .flat_map(|(entity, arena)| {
            arena
                .iter_context_ids()
                .filter(|id| {
                    let context = arena.get_context(*id).unwrap();
                    filter(context)
                })
                .map(|id| (*entity, id))
        })
        .collect()
}
