use core::hash::Hasher;
use std::any::TypeId;
use std::hash::Hash;

use crate::asset_saving::{SaveFsm, SaveGraph};
use crate::egui_fsm::lib::FsmUiContext;
use crate::egui_nodes::lib::NodesContext;
use crate::fsm_show::{FsmIndices, FsmReprSpec};
use crate::graph_show::{GraphIndices, GraphReprSpec};
use crate::graph_update::{
    apply_global_changes, convert_fsm_change, convert_graph_change, update_graph_asset, Change,
    FsmChange, FsmPropertiesChange, GlobalChange, GraphChange,
};
use crate::scanner::PersistedAssetHandles;
use crate::tree::{Tree, TreeResult};
use bevy::asset::UntypedAssetId;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use bevy_animation_graph::core::animated_scene::{AnimatedScene, AnimatedSceneInstance};
use bevy_animation_graph::core::animation_clip::EntityPath;
use bevy_animation_graph::core::animation_graph::{AnimationGraph, NodeId, PinId};
use bevy_animation_graph::core::animation_graph_player::AnimationGraphPlayer;
use bevy_animation_graph::core::animation_node::AnimationNode;
use bevy_animation_graph::core::context::{GraphContextId, SpecContext};
use bevy_animation_graph::core::edge_data::AnimationEvent;
use bevy_animation_graph::core::state_machine::high_level::{
    State, StateId, StateMachine, Transition, TransitionId,
};
use bevy_animation_graph::prelude::{ReflectEditProxy, ReflectNodeLike};
use bevy_egui::EguiContext;
use bevy_inspector_egui::reflect_inspector::{Context, InspectorUi};
use bevy_inspector_egui::{bevy_egui, egui};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use egui_notify::{Anchor, Toasts};

use super::editor_windows::debugger::DebuggerWindow;
use super::editor_windows::scene_preview::ScenePreviewWindow;
use super::scenes::PreviewScene;
use super::utils::{self, OrbitView};

pub fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        ui_state.ui(world, egui_context.get_mut())
    });
}

pub struct GraphSelection {
    pub graph: AssetId<AnimationGraph>,
    pub graph_indices: GraphIndices,
    pub nodes_context: NodesContext,
}

pub struct FsmSelection {
    pub fsm: AssetId<StateMachine>,
    pub graph_indices: FsmIndices,
    pub nodes_context: FsmUiContext,
    pub state_creation: State,
    pub transition_creation: Transition,
}

#[derive(Default)]
pub enum InspectorSelection {
    FsmTransition(FsmTransitionSelection),
    FsmState(FsmStateSelection),
    Node(NodeSelection),
    /// The selection data is contained in the GraphSelection, not here. This is because the graph
    /// should not become unselected whenever the inspector window shows something else.
    Graph,
    /// The selection data is contained in the FsmSelection, not here. This is because the FSM
    /// should not become unselected whenever the inspector window shows something else.
    Fsm,
    #[default]
    Nothing,
}

pub struct FsmTransitionSelection {
    pub(crate) fsm: AssetId<StateMachine>,
    pub(crate) state: TransitionId,
}

pub struct FsmStateSelection {
    pub(crate) fsm: AssetId<StateMachine>,
    pub(crate) state: StateId,
}

pub struct NodeSelection {
    pub(crate) graph: AssetId<AnimationGraph>,
    pub(crate) node: NodeId,
    pub(crate) name_buf: String,
    pub(crate) selected_pin_id: Option<PinId>,
}

pub struct SceneSelection {
    pub(crate) scene: Handle<AnimatedScene>,
    pub(crate) active_context: HashMap<UntypedAssetId, GraphContextId>,
    pub(crate) event_table: Vec<AnimationEvent>,
    /// Just here as a buffer for the editor
    pub(crate) event_editor: AnimationEvent,
}

#[derive(Default)]
pub struct NodeCreation {
    node: AnimationNode,
}

#[derive(Default)]
pub struct EditorSelection {
    pub graph_editor: Option<GraphSelection>,
    pub fsm_editor: Option<FsmSelection>,
    pub inspector_selection: InspectorSelection,
    pub(crate) scene: Option<SceneSelection>,
    pub(crate) node_creation: NodeCreation,
    pub(crate) entity_path: Option<EntityPath>,
}

pub enum RequestSave {
    Graph(AssetId<AnimationGraph>),
    Fsm(AssetId<StateMachine>),
}

#[derive(Resource)]
pub struct UiState {
    pub(crate) state: DockState<EguiWindow>,
    pub selection: EditorSelection,
    pub(crate) graph_changes: Vec<GraphChange>,
    pub(crate) global_changes: Vec<GlobalChange>,
    /// Requests to save a graph. These still need confirmation from the user,
    /// and specification of path where graph should be saved.
    pub(crate) save_requests: Vec<RequestSave>,
    /// Save events to be fired as bevy events after Ui system has finished running
    pub(crate) graph_save_events: Vec<SaveGraph>,
    pub(crate) fsm_save_events: Vec<SaveFsm>,
    pub(crate) notifications: Toasts,
}

impl UiState {
    pub fn new() -> Self {
        let mut state = DockState::new(vec![EguiWindow::GraphEditor, EguiWindow::FsmEditor]);
        let tree = state.main_surface_mut();
        let [graph_editor, inspectors] = tree.split_right(
            NodeIndex::root(),
            0.75,
            vec![
                EguiWindow::Inspector,
                EguiWindow::dynamic(DebuggerWindow {
                    orbit_view: OrbitView {
                        distance: 3.,
                        angles: Vec2::ZERO,
                    },
                }),
            ],
        );
        let [_graph_editor, graph_selector] =
            tree.split_left(graph_editor, 0.2, vec![EguiWindow::GraphSelector]);
        let [_graph_selector, scene_selector] =
            tree.split_below(graph_selector, 0.5, vec![EguiWindow::SceneSelector]);
        let [_scene_selector, _fsm_selector] =
            tree.split_below(scene_selector, 0.5, vec![EguiWindow::FsmSelector]);
        let [_node_inspector, preview] = tree.split_above(
            inspectors,
            0.5,
            vec![
                EguiWindow::dynamic(ScenePreviewWindow {
                    orbit_view: OrbitView {
                        distance: 3.,
                        angles: Vec2::ZERO,
                    },
                }),
                EguiWindow::PreviewHierarchy,
            ],
        );
        let [_preview, _preview_errors] = tree.split_below(
            preview,
            0.8,
            vec![EguiWindow::EventSender, EguiWindow::PreviewErrors],
        );

        Self {
            state,
            selection: EditorSelection::default(),
            graph_changes: vec![],
            global_changes: vec![],
            save_requests: vec![],
            notifications: Toasts::new()
                .with_anchor(Anchor::BottomRight)
                .with_default_font(egui::FontId::proportional(12.)),
            graph_save_events: vec![],
            fsm_save_events: vec![],
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        for save_request in self.save_requests.drain(..) {
            self.state
                .add_window(vec![utils::create_saver_window(world, save_request)]);
        }
        let mut tab_viewer = TabViewer {
            world,
            selection: &mut self.selection,
            graph_changes: &mut self.graph_changes,
            global_changes: &mut self.global_changes,
            save_requests: &mut self.save_requests,
            notifications: &mut self.notifications,
            graph_save_events: &mut self.graph_save_events,
            fsm_save_events: &mut self.fsm_save_events,
        };
        DockArea::new(&mut self.state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);

        self.notifications.show(ctx);
    }
}

pub struct EditorContext<'a> {
    pub selection: &'a mut EditorSelection,
    #[allow(dead_code)] // Temporary while I migrate to extension pattern
    pub global_changes: &'a mut Vec<GlobalChange>,
}

pub trait EditorWindowExtension: std::fmt::Debug + Send + Sync + 'static {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorContext);
    fn display_name(&self) -> String;
}

#[derive(Debug)]
pub struct EditorWindow {
    window: Box<dyn EditorWindowExtension>,
}

#[derive(Debug)]
pub enum EguiWindow {
    GraphEditor,
    FsmEditor,
    PreviewHierarchy,
    PreviewErrors,
    GraphSelector,
    SceneSelector,
    FsmSelector,
    Inspector,
    EventSender,
    GraphSaver(AssetId<AnimationGraph>, String, bool),
    FsmSaver(AssetId<StateMachine>, String, bool),
    DynWindow(EditorWindow),
}

impl EguiWindow {
    pub fn display_name(&self) -> String {
        match self {
            EguiWindow::GraphEditor => "Graph Editor".into(),
            EguiWindow::PreviewHierarchy => "Preview Hierarchy".into(),
            EguiWindow::PreviewErrors => "Errors".into(),
            EguiWindow::GraphSelector => "Select Graph".into(),
            EguiWindow::SceneSelector => "Select Scene".into(),
            EguiWindow::FsmSelector => "Select FSM".into(),
            EguiWindow::GraphSaver(_, _, _) => "Save Graph".into(),
            EguiWindow::FsmSaver(_, _, _) => "Save State Machine".into(),
            EguiWindow::FsmEditor => "FSM Editor".into(),
            EguiWindow::Inspector => "Inspector".into(),
            EguiWindow::EventSender => "Send events".into(),
            EguiWindow::DynWindow(editor_window) => editor_window.window.display_name(),
        }
    }

    pub fn dynamic(ext: impl EditorWindowExtension) -> Self {
        EguiWindow::DynWindow(EditorWindow {
            window: Box::new(ext),
        })
    }
}

struct TabViewer<'a> {
    world: &'a mut World,
    selection: &'a mut EditorSelection,
    graph_changes: &'a mut Vec<GraphChange>,
    global_changes: &'a mut Vec<GlobalChange>,
    save_requests: &'a mut Vec<RequestSave>,
    graph_save_events: &'a mut Vec<SaveGraph>,
    fsm_save_events: &'a mut Vec<SaveFsm>,
    notifications: &'a mut Toasts,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, window: &mut Self::Tab) {
        match window {
            EguiWindow::GraphSelector => Self::graph_selector(self.world, ui, self.selection),
            EguiWindow::SceneSelector => Self::scene_selector(self.world, ui, self.selection),
            EguiWindow::FsmSelector => Self::fsm_selector(self.world, ui, self.selection),
            EguiWindow::GraphEditor => {
                Self::graph_editor(
                    self.world,
                    ui,
                    self.selection,
                    self.graph_changes,
                    self.save_requests,
                );
            }
            EguiWindow::GraphSaver(graph, path, done) => {
                Self::graph_saver(ui, self.graph_save_events, *graph, path, done);
            }
            EguiWindow::FsmSaver(fsm, path, done) => {
                Self::fsm_saver(ui, self.fsm_save_events, *fsm, path, done);
            }
            EguiWindow::PreviewErrors => {
                Self::scene_preview_errors(self.world, ui, self.selection);
            }
            EguiWindow::PreviewHierarchy => {
                Self::scene_preview_entity_tree(self.world, ui, self.selection, self.notifications)
            }
            EguiWindow::FsmEditor => {
                Self::fsm_editor(
                    self.world,
                    ui,
                    self.selection,
                    self.global_changes,
                    self.save_requests,
                );
            }
            EguiWindow::Inspector => match &self.selection.inspector_selection {
                InspectorSelection::FsmTransition(_) => {
                    Self::transition_inspector(self.world, ui, self.selection, self.global_changes)
                }
                InspectorSelection::FsmState(_) => {
                    Self::state_inspector(self.world, ui, self.selection, self.global_changes)
                }
                InspectorSelection::Nothing => {}
                InspectorSelection::Node(_) => {
                    Self::node_inspector(self.world, ui, self.selection, self.graph_changes)
                }
                InspectorSelection::Graph => {
                    Self::graph_inspector(self.world, ui, self.selection, self.graph_changes)
                }
                InspectorSelection::Fsm => {
                    Self::fsm_inspector(self.world, ui, self.selection, self.global_changes)
                }
            },
            EguiWindow::EventSender => Self::event_sender(self.world, ui, self.selection),
            EguiWindow::DynWindow(editor_window) => editor_window.window.ui(
                ui,
                self.world,
                &mut EditorContext {
                    selection: self.selection,
                    global_changes: self.global_changes,
                },
            ),
        }

        while !self.graph_changes.is_empty() {
            let must_regen_indices = self.world.resource_scope::<Assets<AnimationGraph>, _>(
                |world, mut graph_assets| {
                    world.resource_scope::<Assets<StateMachine>, _>(|_, fsm_assets| {
                        update_graph_asset(
                            self.graph_changes.clone(),
                            &mut graph_assets,
                            &fsm_assets,
                        )
                    })
                },
            );
            self.graph_changes.clear();
            if must_regen_indices {
                if let Some(graph_selection) = self.selection.graph_editor.as_mut() {
                    graph_selection.graph_indices =
                        utils::update_graph_indices(self.world, graph_selection.graph);
                }
            }
        }

        let must_regen_indices = apply_global_changes(self.world, self.global_changes.clone());
        if must_regen_indices {
            if let Some(graph_selection) = self.selection.graph_editor.as_mut() {
                graph_selection.graph_indices =
                    utils::update_graph_indices(self.world, graph_selection.graph);
            }
            if let Some(fsm_selection) = self.selection.fsm_editor.as_mut() {
                fsm_selection.graph_indices =
                    utils::update_fsm_indices(self.world, fsm_selection.fsm);
            }
        }
        self.global_changes.clear();
    }

    fn force_close(&mut self, tab: &mut Self::Tab) -> bool {
        matches!(
            tab,
            EguiWindow::GraphSaver(_, _, true) | EguiWindow::FsmSaver(_, _, true)
        )
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        window.display_name().into()
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        matches!(
            tab,
            EguiWindow::GraphSaver(_, _, _) | EguiWindow::FsmSaver(_, _, _)
        )
    }
}

/// Ui functions
impl TabViewer<'_> {
    fn graph_saver(
        ui: &mut egui::Ui,
        save_events: &mut Vec<SaveGraph>,
        graph_id: AssetId<AnimationGraph>,
        path: &mut String,
        done: &mut bool,
    ) {
        ui.label("Save graph as:");
        ui.text_edit_singleline(path);
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            if ui.button("Save").clicked() {
                *done = true;
                save_events.push(SaveGraph {
                    graph: graph_id,
                    virtual_path: path.clone().into(),
                });
            }
        });
    }

    fn fsm_saver(
        ui: &mut egui::Ui,
        save_events: &mut Vec<SaveFsm>,
        fsm_id: AssetId<StateMachine>,
        path: &mut String,
        done: &mut bool,
    ) {
        ui.label("Save state machine as:");
        ui.text_edit_singleline(path);
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            if ui.button("Save").clicked() {
                *done = true;
                save_events.push(SaveFsm {
                    fsm: fsm_id,
                    virtual_path: path.clone().into(),
                });
            }
        });
    }

    fn scene_preview_entity_tree(
        world: &mut World,
        ui: &mut egui::Ui,
        selection: &mut EditorSelection,
        notifications: &mut Toasts,
    ) {
        if selection.scene.is_none() {
            return;
        };
        let mut query = world.query::<(&AnimatedSceneInstance, &PreviewScene)>();
        let Ok((instance, _)) = query.get_single(world) else {
            return;
        };
        let entity = instance.player_entity;
        let tree = Tree::entity_tree(world, entity);
        match utils::select_from_branches(ui, tree.0) {
            TreeResult::Leaf((_, path)) => {
                let path = EntityPath { parts: path };
                ui.output_mut(|o| o.copied_text = path.to_slashed_string());
                notifications.info(format!("{} copied to clipboard", path.to_slashed_string()));
                selection.entity_path = Some(path);
            }
            TreeResult::Node((_, path)) => {
                let path = EntityPath { parts: path };
                ui.output_mut(|o| o.copied_text = path.to_slashed_string());
                notifications.info(format!("{} copied to clipboard", path.to_slashed_string()));
                selection.entity_path = Some(path);
            }
            TreeResult::None => (),
        }
    }

    fn graph_editor(
        world: &mut World,
        ui: &mut egui::Ui,
        selection: &mut EditorSelection,
        graph_changes: &mut Vec<GraphChange>,
        save_requests: &mut Vec<RequestSave>,
    ) {
        let Some(graph_selection) = &mut selection.graph_editor else {
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
                        &mut selection.scene,
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

                    let maybe_graph_context = selection
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
                graph_changes.extend(changes);

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
                            &mut selection.inspector_selection
                        {
                            if &node_selection.node != node_name
                                || node_selection.graph != graph_selection.graph
                            {
                                node_selection.node.clone_from(node_name);
                                node_selection.name_buf.clone_from(node_name);
                                node_selection.graph = graph_selection.graph;
                            }
                        } else if graph_selection.nodes_context.is_node_just_selected() {
                            selection.inspector_selection =
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
                save_requests.push(RequestSave::Graph(graph_selection.graph));
            }
        });
        // ----------------------------------------------------------------
    }

    fn event_sender(world: &mut World, ui: &mut egui::Ui, selection: &mut EditorSelection) {
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
        let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

        let Some(scene_selection) = &mut selection.scene else {
            return;
        };
        let Some(graph_player) =
            utils::get_animation_graph_player_mut(unsafe { unsafe_world.world_mut() })
        else {
            return;
        };

        ui.horizontal_wrapped(|ui| {
            scene_selection.event_table.retain(|ev| {
                egui::Frame::none()
                    .stroke(egui::Stroke::new(1., egui::Color32::WHITE))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button(format!("{:?}", ev)).clicked() {
                                graph_player.send_event(ev.clone());
                            }
                            !ui.button("×").clicked()
                        })
                        .inner
                    })
                    .inner
            });
        });

        ui.separator();

        env.ui_for_reflect(&mut scene_selection.event_editor, ui);
        if ui.button("Add").clicked() {
            scene_selection
                .event_table
                .push(scene_selection.event_editor.clone());
        }
    }

    fn fsm_editor(
        world: &mut World,
        ui: &mut egui::Ui,
        selection: &mut EditorSelection,
        global_changes: &mut Vec<GlobalChange>,
        save_requests: &mut Vec<RequestSave>,
    ) {
        let Some(fsm_selection) = &mut selection.fsm_editor else {
            ui.centered_and_justified(|ui| ui.label("Select a state machine to edit!"));
            return;
        };

        world.resource_scope::<Assets<StateMachine>, ()>(|world, fsm_assets| {
            world.resource_scope::<Assets<AnimationGraph>, ()>(|world, graph_assets| {
                if !fsm_assets.contains(fsm_selection.fsm) {
                    return;
                }

                let changes = {
                    let fsm = fsm_assets.get(fsm_selection.fsm).unwrap();

                    // Autoselect context if none selected and some available
                    if let (Some(scene), Some(available_contexts)) = (
                        &mut selection.scene,
                        utils::list_graph_contexts(world, |ctx| {
                            let graph_id = ctx.get_graph_id();
                            graph_assets
                                .get(graph_id)
                                .map(|graph| graph.contains_state_machine(fsm_selection.fsm))
                                .is_some()
                        }),
                    ) {
                        if scene
                            .active_context
                            .get(&fsm_selection.fsm.untyped())
                            .is_none()
                            && !available_contexts.is_empty()
                        {
                            scene
                                .active_context
                                .insert(fsm_selection.fsm.untyped(), available_contexts[0]);
                        }
                    }

                    let graph_player = utils::get_animation_graph_player(world);

                    let maybe_fsm_state = selection
                        .scene
                        .as_ref()
                        .and_then(|s| s.active_context.get(&fsm_selection.fsm.untyped()))
                        .zip(graph_player)
                        .and_then(|(id, p)| Some(id).zip(p.get_context_arena()))
                        .and_then(|(id, ca)| ca.get_context(*id))
                        .and_then(|ctx| {
                            let graph_id = ctx.get_graph_id();
                            let graph = graph_assets.get(graph_id).unwrap();
                            let node_id = graph.contains_state_machine(fsm_selection.fsm).unwrap();
                            ctx.caches
                                .get_primary(|c| c.get_fsm_state(&node_id).cloned())
                        });

                    let fsm_repr_spec = FsmReprSpec::from_fsm(
                        fsm,
                        &fsm_selection.graph_indices,
                        &fsm_assets,
                        maybe_fsm_state,
                    );

                    fsm_selection.nodes_context.show(
                        fsm_repr_spec.states,
                        fsm_repr_spec.transitions,
                        ui,
                    );
                    fsm_selection.nodes_context.get_changes().clone()
                }
                .into_iter()
                .map(|c| convert_fsm_change(c, &fsm_selection.graph_indices, fsm_selection.fsm));
                global_changes.extend(changes);

                // --- Update selection for state inspector.
                // ----------------------------------------------------------------

                if let Some(selected_node) = fsm_selection
                    .nodes_context
                    .get_selected_states()
                    .iter()
                    .rev()
                    .find(|id| **id > 1)
                {
                    let state_name = fsm_selection
                        .graph_indices
                        .state_indices
                        .name(*selected_node)
                        .unwrap();
                    if fsm_selection.nodes_context.is_node_just_selected() {
                        selection.inspector_selection =
                            InspectorSelection::FsmState(FsmStateSelection {
                                fsm: fsm_selection.fsm,
                                state: state_name.clone(),
                            });
                    }
                }

                if let Some(selected_transition) = fsm_selection
                    .nodes_context
                    .get_selected_transitions()
                    .iter()
                    .next_back()
                {
                    let (_, transition_id, _) = fsm_selection
                        .graph_indices
                        .transition_indices
                        .edge(*selected_transition)
                        .unwrap();
                    if fsm_selection.nodes_context.is_transition_just_selected() {
                        selection.inspector_selection =
                            InspectorSelection::FsmTransition(FsmTransitionSelection {
                                fsm: fsm_selection.fsm,
                                state: transition_id.clone(),
                            });
                    }
                }
                // ----------------------------------------------------------------
            });
        });

        // --- Initiate fsm saving if Ctrl+S pressed
        // ----------------------------------------------------------------
        world.resource_scope::<ButtonInput<KeyCode>, ()>(|_, input| {
            if input.pressed(KeyCode::ControlLeft) && input.just_pressed(KeyCode::KeyS) {
                save_requests.push(RequestSave::Fsm(fsm_selection.fsm));
            }
        });
        // ----------------------------------------------------------------
    }

    /// Display all assets of the specified asset type `A`
    pub fn graph_selector(world: &mut World, ui: &mut egui::Ui, selection: &mut EditorSelection) {
        let mut queue = CommandQueue::default();
        let mut chosen_id: Option<AssetId<AnimationGraph>> = None;

        world.resource_scope::<AssetServer, ()>(|world, asset_server| {
            world.resource_scope::<Assets<AnimationGraph>, ()>(|world, mut graph_assets| {
                let mut assets: Vec<_> = graph_assets.ids().collect();
                assets.sort();
                let paths = assets
                    .into_iter()
                    .map(|id| (utils::handle_path(id.untyped(), &asset_server), id))
                    .collect();
                if let TreeResult::Leaf(id) = utils::path_selector(ui, paths) {
                    chosen_id = Some(id);
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    let mut graph_handles =
                        world.get_resource_mut::<PersistedAssetHandles>().unwrap();
                    if ui.button("New Graph").clicked() {
                        let new_handle = graph_assets.add(AnimationGraph::default());
                        info!("Creating graph with id: {:?}", new_handle.id());
                        graph_handles.unsaved_graphs.insert(new_handle);
                    }
                });
            });
        });
        queue.apply(world);
        if let Some(chosen_id) = chosen_id {
            selection.graph_editor = Some(GraphSelection {
                graph: chosen_id,
                graph_indices: utils::update_graph_indices(world, chosen_id),
                nodes_context: NodesContext::default(),
            });
            selection.inspector_selection = InspectorSelection::Graph;
        }
    }

    pub fn scene_selector(world: &mut World, ui: &mut egui::Ui, selection: &mut EditorSelection) {
        let mut queue = CommandQueue::default();

        let mut chosen_handle: Option<Handle<AnimatedScene>> = None;

        world.resource_scope::<AssetServer, ()>(|world, asset_server| {
            // create a context with access to the world except for the `R` resource
            world.resource_scope::<Assets<AnimatedScene>, ()>(|_, assets| {
                let mut assets: Vec<_> = assets.ids().collect();
                assets.sort();
                let paths = assets
                    .into_iter()
                    .map(|id| (utils::handle_path(id.untyped(), &asset_server), id))
                    .collect();
                let chosen_id = utils::path_selector(ui, paths);
                if let TreeResult::Leaf(id) = chosen_id {
                    chosen_handle = Some(
                        asset_server
                            .get_handle(asset_server.get_path(id).unwrap())
                            .unwrap(),
                    )
                }
            });
        });
        queue.apply(world);

        // TODO: Make sure to clear out all places that hold a graph context id
        //       when changing scene selection.

        if let Some(chosen_handle) = chosen_handle {
            let event_table = if let Some(scn) = &selection.scene {
                scn.event_table.clone()
            } else {
                Vec::new()
            };
            selection.scene = Some(SceneSelection {
                scene: chosen_handle,
                active_context: HashMap::default(),
                event_table,
                event_editor: AnimationEvent::default(),
            });
        }
    }

    pub fn fsm_selector(world: &mut World, ui: &mut egui::Ui, selection: &mut EditorSelection) {
        let mut queue = CommandQueue::default();
        let mut chosen_id: Option<AssetId<StateMachine>> = None;

        world.resource_scope::<AssetServer, ()>(|world, asset_server| {
            world.resource_scope::<Assets<StateMachine>, ()>(|_world, graph_assets| {
                let mut assets: Vec<_> = graph_assets.ids().collect();
                assets.sort();
                let paths = assets
                    .into_iter()
                    .map(|id| (utils::handle_path(id.untyped(), &asset_server), id))
                    .collect();
                if let TreeResult::Leaf(id) = utils::path_selector(ui, paths) {
                    chosen_id = Some(id);
                }
                // ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                //     let mut graph_handles = world.get_resource_mut::<GraphHandles>().unwrap();
                //     CREATE NEW FSM & STUFF
                // });
            });
        });
        queue.apply(world);
        if let Some(chosen_id) = chosen_id {
            selection.fsm_editor = Some(FsmSelection {
                fsm: chosen_id,
                graph_indices: utils::update_fsm_indices(world, chosen_id),
                nodes_context: FsmUiContext::default(),
                state_creation: State::default(),
                transition_creation: Transition::default(),
            });
            selection.inspector_selection = InspectorSelection::Fsm;
        }
    }

    fn graph_inspector(
        world: &mut World,
        ui: &mut egui::Ui,
        selection: &mut EditorSelection,
        graph_changes: &mut Vec<GraphChange>,
    ) {
        ui.heading("Animation graph");

        utils::select_graph_context(world, ui, selection);

        ui.collapsing("Create node", |ui| {
            Self::node_creator(world, ui, selection, graph_changes)
        });

        let mut changes = Vec::new();

        let Some(graph_selection) = &mut selection.graph_editor else {
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

        graph_changes.extend(changes);

        queue.apply(world);
    }

    fn fsm_inspector(
        world: &mut World,
        ui: &mut egui::Ui,
        selection: &mut EditorSelection,
        global_changes: &mut Vec<GlobalChange>,
    ) {
        ui.heading("State machine");
        let mut changes = Vec::new();

        utils::select_graph_context_fsm(world, ui, selection);

        let Some(fsm_selection) = &mut selection.fsm_editor else {
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

        let changed = env.ui_for_reflect_with_options(
            &mut properties,
            ui,
            egui::Id::new(fsm_selection.fsm),
            &(),
        );
        if changed {
            changes.push(GlobalChange::FsmChange {
                asset_id: fsm_selection.fsm,
                change: FsmChange::PropertiesChanged(properties),
            });
        }

        if let Some(new_state) = Self::add_state_ui(ui, fsm_selection, &mut env) {
            changes.push(GlobalChange::FsmChange {
                asset_id: fsm_selection.fsm,
                change: FsmChange::StateAdded(new_state),
            })
        }

        if let Some(transition) = Self::add_transition_ui(ui, fsm_selection, &mut env) {
            changes.push(GlobalChange::FsmChange {
                asset_id: fsm_selection.fsm,
                change: FsmChange::TransitionAdded(transition),
            })
        }

        global_changes.extend(changes);
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

    fn node_creator(
        world: &mut World,
        ui: &mut egui::Ui,
        selection: &mut EditorSelection,
        graph_changes: &mut Vec<GraphChange>,
    ) {
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

        let original_type_id = selection.node_creation.node.inner.type_id();
        let mut type_id = original_type_id;
        egui::Grid::new("node creator fields")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut selection.node_creation.node.name);
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
                                let padding =
                                    " ".repeat(longest_short_name - node_type.short.len());
                                let name =
                                    format!("{}{padding}  {}", node_type.short, node_type.path);
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
                        selection.node_creation.node.inner.as_partial_reflect_mut(),
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
                selection.node_creation.node.inner = inner;
                Ok::<_, &str>(())
            })();

            if let Err(err) = result {
                warn!("Failed to start creating node of type {type_id:?}: {err}");
            }
        }

        let submit_response = ui.button("Create node");

        if submit_response.clicked() && selection.graph_editor.is_some() {
            let graph_selection = selection.graph_editor.as_ref().unwrap();
            graph_changes.push(GraphChange {
                change: Change::NodeCreated(selection.node_creation.node.clone()),
                graph: graph_selection.graph,
            });
        }

        queue.apply(world);
    }

    fn node_inspector(
        world: &mut World,
        ui: &mut egui::Ui,
        selection: &mut EditorSelection,
        graph_changes: &mut Vec<GraphChange>,
    ) {
        ui.heading("Graph node");

        let mut changes = Vec::new();

        let InspectorSelection::Node(node_selection) = &mut selection.inspector_selection else {
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
            selection.inspector_selection = InspectorSelection::Nothing;
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

        graph_changes.extend(changes);

        queue.apply(world);
    }

    fn state_inspector(
        world: &mut World,
        ui: &mut egui::Ui,
        selection: &mut EditorSelection,
        graph_changes: &mut Vec<GlobalChange>,
    ) {
        ui.heading("FSM State");

        let mut changes = Vec::new();

        let Some(_fsm_selection) = &mut selection.fsm_editor else {
            return;
        };

        let InspectorSelection::FsmState(state_selection) = &mut selection.inspector_selection
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
            selection.inspector_selection = InspectorSelection::Nothing;
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

        graph_changes.extend(changes);

        queue.apply(world);
    }

    fn transition_inspector(
        world: &mut World,
        ui: &mut egui::Ui,
        selection: &mut EditorSelection,
        global_changes: &mut Vec<GlobalChange>,
    ) {
        ui.heading("FSM Transition");
        let mut changes = Vec::new();

        let Some(fsm_selection) = &mut selection.fsm_editor else {
            return;
        };

        let InspectorSelection::FsmTransition(transition_selection) =
            &mut selection.inspector_selection
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
            selection.inspector_selection = InspectorSelection::Nothing;
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

        global_changes.extend(changes);

        queue.apply(world);
    }

    fn scene_preview_errors(world: &mut World, ui: &mut egui::Ui, selection: &mut EditorSelection) {
        if selection.scene.is_none() {
            return;
        };
        let mut query = world.query::<(&AnimatedSceneInstance, &PreviewScene)>();
        let Ok((instance, _)) = query.get_single(world) else {
            return;
        };
        let entity = instance.player_entity;
        let mut query = world.query::<&AnimationGraphPlayer>();
        let Ok(player) = query.get(world, entity) else {
            return;
        };
        if let Some(error) = player.get_error() {
            ui.horizontal(|ui| {
                ui.label("⚠");
                ui.label(format!("{}", error));
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No errors to show");
            });
        }
    }
}
