use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::With,
    world::{CommandQueue, World},
};
use egui_dock::{DockArea, DockState, NodeIndex};
use egui_notify::Toasts;

use crate::ui::{
    actions::{EditorAction, PushQueue},
    core::{Buffers, EditorWindowExtension, EguiWindow, GlobalState, LegacyEditorWindowContext},
    editor_windows::{
        animation_clip_preview::ClipPreviewWindow, debugger::DebuggerWindow,
        event_sender::EventSenderWindow, event_track_editor::EventTrackEditorWindow,
        fsm_editor::FsmEditorWindow, fsm_selector::FsmSelectorWindow,
        graph_editor::GraphEditorWindow, graph_selector::GraphSelectorWindow,
        inspector::InspectorWindow, preview_hierarchy::PreviewHierarchyWindow,
        ragdoll_editor::RagdollEditorWindow, scene_preview::ScenePreviewWindow,
        scene_preview_errors::ScenePreviewErrorsWindow, scene_selector::SceneSelectorWindow,
        skeleton_preview::SkeletonCollidersPreviewWindow,
    },
    native_windows::{
        EditorWindowContext, NativeEditorWindow, NativeEditorWindowExtension,
        scene_picker::ScenePickerWindow,
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

    pub fn ui() {}
}

fn ragdoll_view(world: &mut World, windows: &mut Windows) -> DockState<EguiWindow> {
    let preview_window = windows.open(RagdollEditorWindow::default());

    let state = DockState::new(vec![preview_window.into()]);

    state
}

fn test_view(world: &mut World) -> DockState<EguiWindow> {
    let scene_picker = NativeEditorWindow::create(world, ScenePickerWindow);
    let state = DockState::new(vec![scene_picker.into()]);

    state
}

fn event_track_view(world: &mut World, windows: &mut Windows) -> DockState<EguiWindow> {
    let event_track_window = windows.open(EventTrackEditorWindow::default());
    let clip_preview_window = windows.open(ClipPreviewWindow::default());

    let mut state = DockState::new(vec![event_track_window.into()]);

    let tree = state.main_surface_mut();

    tree.split_above(NodeIndex::root(), 0.5, vec![clip_preview_window.into()]);

    state
}

fn main_view(world: &mut World, windows: &mut Windows) -> DockState<EguiWindow> {
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

    state
}

fn skeleton_colliders_view(windows: &mut Windows) -> DockState<EguiWindow> {
    let preview_window = windows.open(SkeletonCollidersPreviewWindow::default());

    let state = DockState::new(vec![preview_window.into()]);

    state
}

pub struct EditorViewContext<'a> {
    pub view_entity: Entity,
    pub notifications: &'a mut Toasts,
    pub command_queue: &'a mut CommandQueue,
    pub buffers: &'a mut Buffers,

    // For legacy windows
    pub global_state: &'a mut GlobalState,
    pub windows: &'a mut Windows,
    pub editor_actions: &'a mut PushQueue<EditorAction>,
}

pub struct EditorViewUiState {
    pub entity: Entity,
    pub dock_state: DockState<EguiWindow>,
}

impl EditorViewUiState {
    pub fn init(
        world: &mut World,
        name: impl Into<String>,
        dock_state: DockState<EguiWindow>,
    ) -> Self {
        let entity = EditorView::init(world, name);
        Self { entity, dock_state }
    }

    pub fn ui(&mut self, ctx: &mut egui::Context, world: &mut World, context: EditorViewContext) {
        let mut tab_viewer = TabViewer { world, context };

        DockArea::new(&mut self.dock_state)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .id(egui::Id::new(self.entity))
            .show(ctx, &mut tab_viewer);
    }

    pub fn ragdoll(world: &mut World, windows: &mut Windows, name: impl Into<String>) -> Self {
        let dock_state = ragdoll_view(world, windows);
        Self::init(world, name, dock_state)
    }

    pub fn test(world: &mut World, _windows: &mut Windows, name: impl Into<String>) -> Self {
        let dock_state = test_view(world);
        Self::init(world, name, dock_state)
    }

    pub fn event_tracks(world: &mut World, windows: &mut Windows, name: impl Into<String>) -> Self {
        let dock_state = event_track_view(world, windows);
        Self::init(world, name, dock_state)
    }

    pub fn animation_graphs(
        world: &mut World,
        windows: &mut Windows,
        name: impl Into<String>,
    ) -> Self {
        let dock_state = main_view(world, windows);
        Self::init(world, name, dock_state)
    }

    pub fn skeleton_colliders(
        world: &mut World,
        windows: &mut Windows,
        name: impl Into<String>,
    ) -> Self {
        let dock_state = skeleton_colliders_view(windows);
        Self::init(world, name, dock_state)
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
                    .with_window_mut(*editor_window, |window, windows| {
                        let mut ctx = LegacyEditorWindowContext {
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
