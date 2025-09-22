use std::any::Any;

use crate::UiCamera;
use crate::egui_fsm::lib::FsmUiContext;
use crate::egui_nodes::lib::NodesContext;
use crate::ui::editor_windows::ragdoll_editor::RagdollEditorWindow;
use bevy::asset::UntypedAssetId;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_animation_graph::core::animated_scene::AnimatedScene;
use bevy_animation_graph::core::animation_clip::EntityPath;
use bevy_animation_graph::core::animation_graph::{AnimationGraph, NodeId, PinId};
use bevy_animation_graph::core::animation_node::AnimationNode;
use bevy_animation_graph::core::context::GraphContextId;
use bevy_animation_graph::core::edge_data::AnimationEvent;
use bevy_animation_graph::core::state_machine::high_level::{
    State, StateId, StateMachine, Transition, TransitionId,
};
use bevy_egui::{EguiContext, PrimaryEguiContext};
use bevy_inspector_egui::{bevy_egui, egui};
use egui_dock::egui::Color32;
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use egui_notify::{Anchor, Toasts};

use super::actions::saving::SaveAction;
use super::actions::{EditorAction, PendingActions, PushQueue};
use super::editor_windows::animation_clip_preview::ClipPreviewWindow;
use super::editor_windows::debugger::DebuggerWindow;
use super::editor_windows::event_sender::EventSenderWindow;
use super::editor_windows::event_track_editor::EventTrackEditorWindow;
use super::editor_windows::fsm_editor::FsmEditorWindow;
use super::editor_windows::fsm_selector::FsmSelectorWindow;
use super::editor_windows::graph_editor::GraphEditorWindow;
use super::editor_windows::graph_selector::GraphSelectorWindow;
use super::editor_windows::inspector::InspectorWindow;
use super::editor_windows::preview_hierarchy::PreviewHierarchyWindow;
use super::editor_windows::scene_preview::ScenePreviewWindow;
use super::editor_windows::scene_preview_errors::ScenePreviewErrorsWindow;
use super::editor_windows::scene_selector::SceneSelectorWindow;

use super::editor_windows::skeleton_preview::SkeletonCollidersPreviewWindow;
pub use super::windows::EditorWindowExtension;
use super::windows::{WindowId, Windows};

#[derive(Component)]
pub struct HasLoadedImgLoaders;
pub fn setup_ui(
    mut egui_context: Query<(Entity, &mut EguiContext), Without<HasLoadedImgLoaders>>,
    mut commands: Commands,
) {
    for (entity, mut ctx) in &mut egui_context {
        egui_extras::install_image_loaders(ctx.get_mut());
        commands.entity(entity).insert(HasLoadedImgLoaders);
    }
}

pub fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        world.resource_scope::<PendingActions, _>(|world, mut pending_actions| {
            ui_state.ui(world, egui_context.get_mut(), &mut pending_actions)
        });
    });
}

pub struct GraphSelection {
    pub graph: Handle<AnimationGraph>,
    pub nodes_context: NodesContext,
}

pub struct FsmSelection {
    pub fsm: Handle<StateMachine>,
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
    pub(crate) fsm: Handle<StateMachine>,
    pub(crate) state: TransitionId,
}

pub struct FsmStateSelection {
    pub(crate) fsm: Handle<StateMachine>,
    pub(crate) state: StateId,
}

pub struct NodeSelection {
    pub(crate) graph: Handle<AnimationGraph>,
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
    pub(crate) node_type_search: String,
    pub(crate) node: AnimationNode,
}

#[derive(Default)]
pub struct GlobalState {
    pub graph_editor: Option<GraphSelection>,
    pub fsm_editor: Option<FsmSelection>,
    pub inspector_selection: InspectorSelection,
    pub(crate) scene: Option<SceneSelection>,
    pub(crate) node_creation: NodeCreation,
    pub(crate) entity_path: Option<EntityPath>,
}

pub struct ViewState {
    pub(crate) name: String,
    pub(crate) dock_state: DockState<EguiWindow>,
}

impl ViewState {
    pub fn ui(&mut self, ctx: &mut egui::Context, world: &mut World, context: EditorViewContext) {
        let mut tab_viewer = TabViewer { world, context };

        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .id(egui::Id::new(self.name.clone()))
            .show(ctx, &mut tab_viewer);
    }

    #[allow(dead_code)]
    pub fn empty(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dock_state: DockState::new(vec![]),
        }
    }

    pub fn event_track_view(windows: &mut Windows, name: impl Into<String>) -> Self {
        let event_track_window = windows.open(EventTrackEditorWindow::default());
        let clip_preview_window = windows.open(ClipPreviewWindow::default());

        let mut state = DockState::new(vec![event_track_window.into()]);

        let tree = state.main_surface_mut();

        tree.split_above(NodeIndex::root(), 0.5, vec![clip_preview_window.into()]);

        Self {
            name: name.into(),
            dock_state: state,
        }
    }

    pub fn skeleton_colliders_view(windows: &mut Windows, name: impl Into<String>) -> Self {
        let preview_window = windows.open(SkeletonCollidersPreviewWindow::default());

        let state = DockState::new(vec![preview_window.into()]);

        Self {
            name: name.into(),
            dock_state: state,
        }
    }

    pub fn ragdoll_view(windows: &mut Windows, name: impl Into<String>) -> Self {
        let preview_window = windows.open(RagdollEditorWindow::default());

        let state = DockState::new(vec![preview_window.into()]);

        Self {
            name: name.into(),
            dock_state: state,
        }
    }

    pub fn main_view(windows: &mut Windows, name: impl Into<String>) -> Self {
        let graph_editor = windows.open(GraphEditorWindow);
        let fsm_editor = windows.open(FsmEditorWindow);
        let inspector = windows.open(InspectorWindow);
        let debugger = windows.open(DebuggerWindow::default());
        let graph_selector = windows.open(GraphSelectorWindow);
        let scene_selector = windows.open(SceneSelectorWindow);
        let fsm_selector = windows.open(FsmSelectorWindow);
        let scene_preview = windows.open(ScenePreviewWindow::default());
        let preview_hierarchy = windows.open(PreviewHierarchyWindow);
        let event_sender = windows.open(EventSenderWindow);
        let scene_preview_errors = windows.open(ScenePreviewErrorsWindow);

        let mut state = DockState::new(vec![graph_editor.into(), fsm_editor.into()]);
        let tree = state.main_surface_mut();
        let [graph_editor, inspectors] = tree.split_right(
            NodeIndex::root(),
            0.75,
            vec![inspector.into(), debugger.into()],
        );
        let [_graph_editor, graph_selector] =
            tree.split_left(graph_editor, 0.2, vec![graph_selector.into()]);
        let [_graph_selector, scene_selector] =
            tree.split_below(graph_selector, 0.5, vec![scene_selector.into()]);
        let [_scene_selector, _fsm_selector] =
            tree.split_below(scene_selector, 0.5, vec![fsm_selector.into()]);
        let [_node_inspector, preview] = tree.split_above(
            inspectors,
            0.5,
            vec![scene_preview.into(), preview_hierarchy.into()],
        );
        let [_preview, _preview_errors] = tree.split_below(
            preview,
            0.8,
            vec![event_sender.into(), scene_preview_errors.into()],
        );

        Self {
            name: name.into(),
            dock_state: state,
        }
    }
}

#[derive(Resource)]
pub struct UiState {
    pub global_state: GlobalState,

    pub(crate) windows: Windows,

    pub(crate) notifications: Toasts,

    pub(crate) views: Vec<ViewState>,
    pub(crate) active_view: Option<usize>,
}

impl UiState {
    pub fn new() -> Self {
        let mut windows = Windows::default();

        Self {
            global_state: GlobalState::default(),
            notifications: Toasts::new()
                .with_anchor(Anchor::BottomRight)
                .with_default_font(egui::FontId::proportional(12.)),

            views: vec![
                ViewState::main_view(&mut windows, "Graph editing"),
                ViewState::event_track_view(&mut windows, "Event tracks"),
                ViewState::skeleton_colliders_view(&mut windows, "Skeleton colliders"),
                ViewState::ragdoll_view(&mut windows, "Ragdoll"),
            ],
            active_view: Some(0),
            windows,
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context, queue: &mut PendingActions) {
        if let Some(view_action) = view_selection_bar(ctx, self) {
            queue.actions.push(EditorAction::View(view_action));
        }

        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            queue
                .actions
                .push(EditorAction::Save(SaveAction::RequestMultiple));
        }

        let view_context = EditorViewContext {
            global_state: &mut self.global_state,
            windows: &mut self.windows,
            notifications: &mut self.notifications,
            editor_actions: &mut queue.actions,
        };

        if let Some(active_view_idx) = self.active_view {
            let active_view = &mut self.views[active_view_idx];
            active_view.ui(ctx, world, view_context);
        }

        self.notifications.show(ctx);
    }
}

pub struct EditorViewContext<'a> {
    pub global_state: &'a mut GlobalState,
    pub windows: &'a mut Windows,
    pub notifications: &'a mut Toasts,
    pub editor_actions: &'a mut PushQueue<EditorAction>,
}

pub struct EditorWindowContext<'a> {
    pub window_id: WindowId,
    /// Read-only view on windows other than the current one
    pub windows: &'a Windows,
    pub global_state: &'a mut GlobalState,
    #[allow(dead_code)] // Temporary while I migrate to extension pattern
    pub notifications: &'a mut Toasts,
    pub editor_actions: &'a mut PushQueue<EditorAction>,
}

impl EditorWindowContext<'_> {
    pub fn window_action(&mut self, event: impl Any + Send + Sync) {
        self.editor_actions.window(self.window_id, event)
    }
}

#[derive(Debug)]
pub enum EguiWindow {
    DynWindow(WindowId),
}

impl From<WindowId> for EguiWindow {
    fn from(id: WindowId) -> Self {
        EguiWindow::DynWindow(id)
    }
}

impl EguiWindow {
    pub fn display_name(&self, windows: &Windows) -> String {
        match self {
            EguiWindow::DynWindow(id) => windows.name_for_window(*id),
        }
    }
}

struct TabViewer<'a> {
    world: &'a mut World,
    context: EditorViewContext<'a>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn ui(&mut self, ui: &mut egui::Ui, window: &mut Self::Tab) {
        match window {
            EguiWindow::DynWindow(editor_window) => {
                self.context
                    .windows
                    .with_window_mut(*editor_window, |window, windows| {
                        let mut ctx = EditorWindowContext {
                            window_id: *editor_window,
                            global_state: self.context.global_state,
                            notifications: self.context.notifications,
                            editor_actions: self.context.editor_actions,
                            windows,
                        };

                        ui.push_id(ui.id().with(*editor_window), |ui| {
                            window.ui(ui, self.world, &mut ctx);
                        })
                    });
            }
        }
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui::WidgetText {
        window.display_name(self.context.windows).into()
    }

    fn force_close(&mut self, EguiWindow::DynWindow(window_id): &mut Self::Tab) -> bool {
        !self.context.windows.window_exists(*window_id)
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        match tab {
            EguiWindow::DynWindow(window_id) => self
                .context
                .windows
                .get_window(*window_id)
                .is_some_and(|w| w.closeable()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ViewAction {
    Close(usize),
    Select(usize),
    New(String),
}

fn view_selection_bar(ctx: &mut egui::Context, ui_state: &UiState) -> Option<ViewAction> {
    egui::TopBottomPanel::top("View selector")
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut action = None;
                for (i, view) in ui_state.views.iter().enumerate() {
                    action = action.or(view_button(
                        ui,
                        i,
                        view,
                        ui_state
                            .active_view
                            .as_ref()
                            .map(|active_view| *active_view == i)
                            .unwrap_or(false),
                    ));
                }

                if ui.add(egui::Button::new("➕").frame(false)).clicked() {
                    action = Some(ViewAction::New("new".into()));
                }

                action
            })
            .inner
        })
        .inner
}

fn view_button(
    ui: &mut egui::Ui,
    index: usize,
    view: &ViewState,
    is_selected: bool,
) -> Option<ViewAction> {
    egui::Frame::NONE
        .stroke((1., egui::Color32::DARK_GRAY))
        .corner_radius(egui::CornerRadius {
            nw: 3,
            ne: 3,
            sw: 0,
            se: 0,
        })
        .inner_margin(2.)
        .outer_margin(egui::Margin {
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
        })
        .fill(if is_selected {
            Color32::DARK_GRAY
        } else {
            Color32::TRANSPARENT
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let select_response = ui.add(egui::Button::new(&view.name).frame(false));
                let close_response = ui.add(egui::Button::new("❌").frame(false));

                if select_response.clicked() {
                    Some(ViewAction::Select(index))
                } else if close_response.clicked() {
                    Some(ViewAction::Close(index))
                } else {
                    None
                }
            })
            .inner
        })
        .inner
}
