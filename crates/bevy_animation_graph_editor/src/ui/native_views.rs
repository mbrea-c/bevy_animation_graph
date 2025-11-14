use bevy::ecs::{
    component::Component,
    entity::Entity,
    world::{CommandQueue, World},
};
use egui_dock::{DockArea, DockState, NodeIndex};
use egui_notify::Toasts;

use crate::ui::{
    actions::{EditorAction, PushQueue},
    core::{Buffers, EditorWindowExtension, EguiWindow, LegacyEditorWindowContext},
    editor_windows::ragdoll_editor::RagdollEditorWindow,
    native_windows::{
        EditorWindowContext, NativeEditorWindow, NativeEditorWindowExtension,
        animation_clip_preview::ClipPreviewWindow, debugger::DebuggerWindow,
        event_sender::EventSenderWindow, event_track_editor::EventTrackEditorWindow,
        fsm_editor::FsmEditorWindow, fsm_picker::FsmPickerWindow, graph_editor::GraphEditorWindow,
        graph_picker::GraphPickerWindow, inspector::InspectorWindow,
        preview_hierarchy::PreviewHierarchyWindow, scene_picker::ScenePickerWindow,
        scene_preview::ScenePreviewWindow, scene_preview_errors::ScenePreviewErrorsWindow,
    },
    windows::Windows,
};

#[derive(Component)]
pub struct EditorViewState;

#[derive(Component)]
pub struct EditorView {
    pub(crate) name: String,
}

impl EditorView {
    pub fn init(world: &mut World, name: impl Into<String>) -> Entity {
        world
            .spawn((EditorViewState, EditorView { name: name.into() }))
            .id()
    }
}

fn ragdoll_view(
    _world: &mut World,
    windows: &mut Windows,
    _view_entity: Entity,
) -> DockState<EguiWindow> {
    let preview_window = windows.open(RagdollEditorWindow::default());

    DockState::new(vec![preview_window.into()])
}

fn event_track_view(world: &mut World, view_entity: Entity) -> DockState<EguiWindow> {
    let event_track_window = NativeEditorWindow::create(world, view_entity, EventTrackEditorWindow);
    let clip_preview_window = NativeEditorWindow::create(world, view_entity, ClipPreviewWindow);

    let mut state = DockState::new(vec![event_track_window.into()]);

    let tree = state.main_surface_mut();

    tree.split_above(NodeIndex::root(), 0.5, vec![clip_preview_window.into()]);

    state
}

fn main_view(world: &mut World, view_entity: Entity) -> DockState<EguiWindow> {
    let graph_editor = NativeEditorWindow::create(world, view_entity, GraphEditorWindow);
    let fsm_editor = NativeEditorWindow::create(world, view_entity, FsmEditorWindow);
    let inspector = NativeEditorWindow::create(world, view_entity, InspectorWindow);
    let debugger = NativeEditorWindow::create(world, view_entity, DebuggerWindow);
    let graph_selector = NativeEditorWindow::create(world, view_entity, GraphPickerWindow);
    let scene_selector = NativeEditorWindow::create(world, view_entity, ScenePickerWindow);
    let fsm_selector = NativeEditorWindow::create(world, view_entity, FsmPickerWindow);
    let scene_preview = NativeEditorWindow::create(world, view_entity, ScenePreviewWindow);
    let preview_hierarchy = NativeEditorWindow::create(world, view_entity, PreviewHierarchyWindow);
    let event_sender = NativeEditorWindow::create(world, view_entity, EventSenderWindow);
    let scene_preview_errors =
        NativeEditorWindow::create(world, view_entity, ScenePreviewErrorsWindow);

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

    state
}

pub struct EditorViewContext<'a> {
    pub view_entity: Entity,
    pub notifications: &'a mut Toasts,
    pub command_queue: &'a mut CommandQueue,
    pub buffers: &'a mut Buffers,

    // For legacy windows
    pub windows: &'a mut Windows,
    pub editor_actions: &'a mut PushQueue<EditorAction>,
}

pub struct EditorViewUiState {
    pub entity: Entity,
    pub dock_state: DockState<EguiWindow>,
}

impl EditorViewUiState {
    pub fn init(entity: Entity, dock_state: DockState<EguiWindow>) -> Self {
        Self { entity, dock_state }
    }

    pub fn ui(&mut self, ctx: &mut egui::Context, world: &mut World, context: EditorViewContext) {
        let mut tab_viewer = TabViewer { world, context };

        DockArea::new(&mut self.dock_state)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .id(egui::Id::new(self.entity))
            .show(ctx, &mut tab_viewer);
    }

    pub fn empty(world: &mut World, name: impl Into<String>) -> Self {
        let entity = EditorView::init(world, name);
        let dock_state = DockState::new(vec![]);
        Self::init(entity, dock_state)
    }

    pub fn ragdoll(world: &mut World, windows: &mut Windows, name: impl Into<String>) -> Self {
        let entity = EditorView::init(world, name);
        let dock_state = ragdoll_view(world, windows, entity);
        Self::init(entity, dock_state)
    }

    pub fn event_tracks(world: &mut World, name: impl Into<String>) -> Self {
        let entity = EditorView::init(world, name);
        let dock_state = event_track_view(world, entity);
        Self::init(entity, dock_state)
    }

    pub fn animation_graphs(world: &mut World, name: impl Into<String>) -> Self {
        let entity = EditorView::init(world, name);
        let dock_state = main_view(world, entity);
        Self::init(entity, dock_state)
    }
}

pub struct TabViewer<'a> {
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
                    .with_window_mut(*editor_window, |window, _windows| {
                        let mut ctx = LegacyEditorWindowContext {
                            window_id: *editor_window,
                            notifications: self.context.notifications,
                            editor_actions: self.context.editor_actions,
                        };

                        ui.push_id(ui.id().with(*editor_window), |ui| {
                            window.ui(ui, self.world, &mut ctx);
                        })
                    });
            }
            EguiWindow::EntityWindow(window) => {
                let mut ctx = EditorWindowContext {
                    window_entity: window.entity,
                    view_entity: self.context.view_entity,
                    notifications: self.context.notifications,
                    command_queue: self.context.command_queue,
                    buffers: self.context.buffers,
                    editor_actions: self.context.editor_actions,
                };

                window.ui(ui, self.world, &mut ctx);
            }
        }
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui::WidgetText {
        window.display_name(self.context.windows).into()
    }

    fn force_close(&mut self, window: &mut Self::Tab) -> bool {
        match window {
            EguiWindow::DynWindow(window_id) => !self.context.windows.window_exists(*window_id),
            EguiWindow::EntityWindow(window) => self.world.get_entity(window.entity).is_err(),
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        match tab {
            EguiWindow::DynWindow(window_id) => self
                .context
                .windows
                .get_window(*window_id)
                .is_some_and(|w| w.closeable()),
            EguiWindow::EntityWindow(window) => window.closeable(),
        }
    }
}
