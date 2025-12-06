use bevy::{
    asset::Assets,
    ecs::entity::Entity,
    prelude::{AppTypeRegistry, World},
};
use bevy_animation_graph::{
    builtin_nodes::fsm_node::FsmNode,
    core::{
        animation_graph::AnimationGraph,
        animation_node::AnimationNode,
        context::{graph_context::GraphState, graph_context_arena::GraphContextId},
        state_machine::high_level::{State, StateMachine, Transition},
    },
};
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui::Widget;
use egui_dock::egui;

use crate::ui::{
    actions::{
        EditorAction,
        fsm::{
            CreateState, CreateTransition, FsmAction, FsmProperties, UpdateProperties, UpdateState,
            UpdateTransition,
        },
        graph::{EditNode, GraphAction, RenameNode, UpdateDefaultData, UpdateGraphSpec},
    },
    generic_widgets::{
        data_value::DataValueWidget, graph_input_pin::GraphInputPinWidget, hashmap::HashMapWidget,
        io_spec::IoSpecWidget,
    },
    native_windows::{EditorWindowContext, NativeEditorWindowExtension},
    node_editors::{ReflectEditable, reflect_editor::ReflectNodeEditor},
    state_management::global::{
        active_fsm::ActiveFsm,
        active_fsm_state::ActiveFsmState,
        active_fsm_transition::ActiveFsmTransition,
        active_graph::ActiveGraph,
        active_graph_context::{ActiveContexts, SetActiveContext},
        active_graph_node::ActiveGraphNode,
        fsm::{SetFsmNodeSpec, SetFsmStartState},
        get_global_state,
        inspector_selection::InspectorSelection,
    },
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
                        node: node.id,
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

    world.resource_scope::<Assets<AnimationGraph>, _>(|_, graph_assets| {
        let graph = graph_assets.get(&active_graph.handle)?;

        let graph_spec_buffer = ctx
            .buffers
            .get_mut_or_insert_with(ui.id().with("graph_spec"), || graph.io_spec.clone());

        let spec_response = IoSpecWidget::new_salted(graph_spec_buffer, "graph_spec_widget")
            .show(ui, |ui, i| {
                GraphInputPinWidget::new_salted(i, "graph input pin edit").ui(ui)
            });

        if spec_response.changed() {
            ctx.editor_actions
                .push(EditorAction::Graph(GraphAction::UpdateGraphSpec(
                    UpdateGraphSpec {
                        graph: active_graph.handle.clone(),
                        new_spec: graph_spec_buffer.clone(),
                    },
                )));
        }

        let default_values_buffer = ctx
            .buffers
            .get_mut_or_insert_with(ui.id().with("graph_default_values"), || {
                graph.default_data.clone()
            });

        ui.heading("Default data");

        let default_values_response =
            HashMapWidget::new_salted(default_values_buffer, "graph_default_values").ui(
                ui,
                |ui, key| ui.text_edit_singleline(key),
                |ui, key| ui.label(key),
                |ui, value| ui.add(DataValueWidget::new_salted(value, "default value widget")),
            );

        if default_values_response.changed() {
            ctx.editor_actions
                .push(EditorAction::Graph(GraphAction::UpdateDefaultData(
                    UpdateDefaultData {
                        graph: active_graph.handle.clone(),
                        input_data: default_values_buffer.clone(),
                    },
                )));
        }

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
        // We should make sure to include the fsm id in the buffer id salt to avoid reusing buffers
        // when active FSM changes
        let buffer_id = |ui: &mut egui::Ui, s: &str| ui.id().with(s).with(active_fsm.handle.id());

        let fsm = fsm_assets.get(&active_fsm.handle)?;
        let spec_buffer = ctx
            .buffers
            .get_mut_or_insert_with(buffer_id(ui, "fsm input spec"), || fsm.node_spec.clone());

        let spec_response = IoSpecWidget::new_salted(spec_buffer, "fsm input spec widget")
            .show(ui, |ui, i| ui.text_edit_singleline(i));

        if spec_response.changed() {
            let new = spec_buffer.clone();
            ctx.trigger(SetFsmNodeSpec {
                fsm: active_fsm.handle.clone(),
                new,
            });
        }

        let start_state_buffer = ctx
            .buffers
            .get_mut_or_insert_with(buffer_id(ui, "fsm start state"), || fsm.start_state.clone());

        let r = ui
            .horizontal(|ui| {
                ui.label("start state:");
                ui.text_edit_singleline(start_state_buffer)
            })
            .inner;

        if r.changed() {
            let new = start_state_buffer.clone();
            ctx.trigger(SetFsmStartState {
                fsm: active_fsm.handle.clone(),
                new,
            });
        }

        using_inspector_env(world, |mut env| {
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
            .get_mut_or_insert_with(ui.id(), Transition::default);
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
        let buffer = ctx.buffers.get_mut_or_insert_with(ui.id(), State::default);
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
            .contains_node_that::<FsmNode>(|n| &n.fsm == &active_fsm.handle)
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
    filter: impl Fn(&GraphState) -> bool,
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
