use crate::asset_saving::{SaveFsm, SaveGraph};
use crate::egui_fsm::lib::FsmUiContext;
use crate::egui_nodes::lib::NodesContext;
use crate::fsm_show::FsmIndices;
use crate::graph_show::GraphIndices;
use crate::graph_update::{apply_global_changes, update_graph_asset, GlobalChange, GraphChange};
use bevy::asset::UntypedAssetId;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use bevy_animation_graph::core::animated_scene::AnimatedScene;
use bevy_animation_graph::core::animation_clip::EntityPath;
use bevy_animation_graph::core::animation_graph::{AnimationGraph, NodeId, PinId};
use bevy_animation_graph::core::animation_node::AnimationNode;
use bevy_animation_graph::core::context::GraphContextId;
use bevy_animation_graph::core::edge_data::AnimationEvent;
use bevy_animation_graph::core::state_machine::high_level::{
    State, StateId, StateMachine, Transition, TransitionId,
};
use bevy_egui::EguiContext;
use bevy_inspector_egui::{bevy_egui, egui};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use egui_notify::{Anchor, Toasts};

use super::editor_windows::debugger::DebuggerWindow;
use super::editor_windows::event_sender::EventSenderWindow;
use super::editor_windows::fsm_editor::FsmEditorWindow;
use super::editor_windows::fsm_selector::FsmSelectorWindow;
use super::editor_windows::graph_editor::GraphEditorWindow;
use super::editor_windows::graph_selector::GraphSelectorWindow;
use super::editor_windows::inspector::InspectorWindow;
use super::editor_windows::preview_hierarchy::PreviewHierarchyWindow;
use super::editor_windows::scene_preview::ScenePreviewWindow;
use super::editor_windows::scene_preview_errors::ScenePreviewErrorsWindow;
use super::editor_windows::scene_selector::SceneSelectorWindow;
use super::utils;

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
    pub(crate) node: AnimationNode,
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
        let mut state = DockState::new(vec![
            EguiWindow::dynamic(GraphEditorWindow),
            EguiWindow::dynamic(FsmEditorWindow),
        ]);
        let tree = state.main_surface_mut();
        let [graph_editor, inspectors] = tree.split_right(
            NodeIndex::root(),
            0.75,
            vec![
                EguiWindow::dynamic(InspectorWindow),
                EguiWindow::dynamic(DebuggerWindow::default()),
            ],
        );
        let [_graph_editor, graph_selector] = tree.split_left(
            graph_editor,
            0.2,
            vec![EguiWindow::dynamic(GraphSelectorWindow)],
        );
        let [_graph_selector, scene_selector] = tree.split_below(
            graph_selector,
            0.5,
            vec![EguiWindow::dynamic(SceneSelectorWindow)],
        );
        let [_scene_selector, _fsm_selector] = tree.split_below(
            scene_selector,
            0.5,
            vec![EguiWindow::dynamic(FsmSelectorWindow)],
        );
        let [_node_inspector, preview] = tree.split_above(
            inspectors,
            0.5,
            vec![
                EguiWindow::dynamic(ScenePreviewWindow::default()),
                EguiWindow::dynamic(PreviewHierarchyWindow),
            ],
        );
        let [_preview, _preview_errors] = tree.split_below(
            preview,
            0.8,
            vec![
                EguiWindow::dynamic(EventSenderWindow),
                EguiWindow::dynamic(ScenePreviewErrorsWindow),
            ],
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
    pub notifications: &'a mut Toasts,
    pub graph_changes: &'a mut Vec<GraphChange>,
    pub save_requests: &'a mut Vec<RequestSave>,
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
    GraphSaver(AssetId<AnimationGraph>, String, bool),
    FsmSaver(AssetId<StateMachine>, String, bool),
    DynWindow(EditorWindow),
}

impl EguiWindow {
    pub fn display_name(&self) -> String {
        match self {
            EguiWindow::GraphSaver(_, _, _) => "Save Graph".into(),
            EguiWindow::FsmSaver(_, _, _) => "Save State Machine".into(),
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
            EguiWindow::GraphSaver(graph, path, done) => {
                Self::graph_saver(ui, self.graph_save_events, *graph, path, done);
            }
            EguiWindow::FsmSaver(fsm, path, done) => {
                Self::fsm_saver(ui, self.fsm_save_events, *fsm, path, done);
            }
            EguiWindow::DynWindow(editor_window) => editor_window.window.ui(
                ui,
                self.world,
                &mut EditorContext {
                    selection: self.selection,
                    global_changes: self.global_changes,
                    notifications: self.notifications,
                    graph_changes: self.graph_changes,
                    save_requests: self.save_requests,
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
}
