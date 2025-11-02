use core::any::TypeId;
use std::any::Any;

use crate::egui_fsm::lib::FsmUiContext;
use crate::egui_nodes::lib::NodesContext;
use crate::ui::ecs_utils::get_view_state;
use crate::ui::global_state;
use crate::ui::native_views::{EditorView, EditorViewContext, EditorViewUiState};
use crate::ui::native_windows::{NativeEditorWindow, NativeEditorWindowExtension};
use bevy::asset::UntypedAssetId;
use bevy::ecs::world::CommandQueue;
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
use egui_notify::{Anchor, Toasts};

use super::actions::saving::SaveAction;
use super::actions::{EditorAction, PendingActions, PushQueue};

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

    let mut command_queue = CommandQueue::default();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        world.resource_scope::<PendingActions, _>(|world, mut pending_actions| {
            ui_state.ui(
                world,
                egui_context.get_mut(),
                &mut pending_actions,
                &mut command_queue,
            )
        });
    });

    command_queue.apply(world);
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
pub struct GlobalState {}

#[derive(Resource)]
pub struct UiState {
    pub global_state: GlobalState,

    pub(crate) windows: Windows,

    pub(crate) notifications: Toasts,

    pub(crate) views: Vec<EditorViewUiState>,
    pub(crate) active_view: Option<usize>,
    pub(crate) buffers: Buffers,
}

impl UiState {
    pub fn init(world: &mut World) {
        let mut ui_state = Self {
            global_state: GlobalState::default(),
            notifications: Toasts::new()
                .with_anchor(Anchor::BottomRight)
                .with_default_font(egui::FontId::proportional(12.)),

            views: Vec::new(),
            active_view: Some(0),
            windows: Windows::default(),
            buffers: Buffers::default(),
        };

        let main_view =
            EditorViewUiState::animation_graphs(world, &mut ui_state.windows, "Graph editing");
        ui_state.new_native_view(main_view);

        let event_tracks =
            EditorViewUiState::event_tracks(world, &mut ui_state.windows, "Event tracks");
        ui_state.new_native_view(event_tracks);

        let skeleton_colliders = EditorViewUiState::skeleton_colliders(
            world,
            &mut ui_state.windows,
            "Skeleton colliders",
        );
        ui_state.new_native_view(skeleton_colliders);

        let ragdoll_view = EditorViewUiState::ragdoll(world, &mut ui_state.windows, "Ragdoll");
        ui_state.new_native_view(ragdoll_view);

        let test_view = EditorViewUiState::test(world, &mut ui_state.windows, "!!! Test");
        ui_state.new_native_view(test_view);

        world.insert_resource(ui_state);
        global_state::GlobalState::init(world);
    }

    pub fn new_native_view(&mut self, state: EditorViewUiState) {
        self.views.push(state);
    }

    fn ui(
        &mut self,
        world: &mut World,
        ctx: &mut egui::Context,
        queue: &mut PendingActions,
        command_queue: &mut CommandQueue,
    ) {
        if let Some(view_action) = view_selection_bar(world, ctx, self) {
            queue.actions.push(EditorAction::View(view_action));
        }

        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            queue
                .actions
                .push(EditorAction::Save(SaveAction::RequestMultiple));
        }

        if let Some(active_view_idx) = self.active_view {
            let view_state = &mut self.views[active_view_idx];
            let view_context = EditorViewContext {
                global_state: &mut self.global_state,
                windows: &mut self.windows,
                notifications: &mut self.notifications,
                buffers: &mut self.buffers,

                editor_actions: &mut queue.actions,
                view_entity: view_state.entity,
                command_queue,
            };
            view_state.ui(ctx, world, view_context);
        }

        self.notifications.show(ctx);
    }
}

pub struct LegacyEditorWindowContext<'a> {
    pub window_id: WindowId,
    /// Read-only view on windows other than the current one
    pub windows: &'a Windows,
    pub global_state: &'a mut GlobalState,
    #[allow(dead_code)] // Temporary while I migrate to extension pattern
    pub notifications: &'a mut Toasts,
    pub editor_actions: &'a mut PushQueue<EditorAction>,
}

impl LegacyEditorWindowContext<'_> {
    pub fn window_action(&mut self, event: impl Any + Send + Sync) {
        self.editor_actions.window(self.window_id, event)
    }
}

#[derive(Debug)]
pub enum EguiWindow {
    DynWindow(WindowId),
    EntityWindow(NativeEditorWindow),
}

impl From<WindowId> for EguiWindow {
    fn from(id: WindowId) -> Self {
        EguiWindow::DynWindow(id)
    }
}

impl From<NativeEditorWindow> for EguiWindow {
    fn from(window: NativeEditorWindow) -> Self {
        EguiWindow::EntityWindow(window)
    }
}

impl EguiWindow {
    pub fn display_name(&self, windows: &Windows) -> String {
        match self {
            EguiWindow::DynWindow(id) => windows.name_for_window(*id),
            EguiWindow::EntityWindow(window) => window.display_name(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ViewAction {
    Close(usize),
    Select(usize),
    New(String),
}

fn view_selection_bar(
    world: &mut World,
    ctx: &mut egui::Context,
    ui_state: &UiState,
) -> Option<ViewAction> {
    egui::TopBottomPanel::top("View selector")
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut action = None;
                for (i, view) in ui_state.views.iter().enumerate() {
                    action = action.or(view_button(
                        ui,
                        world,
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
    world: &mut World,
    index: usize,
    view: &EditorViewUiState,
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
                let name = get_view_state::<EditorView>(world, view.entity)
                    .map_or("<VIEW_NOT_FOUND>".into(), |v| v.name.clone());
                let select_response = ui.add(egui::Button::new(name).frame(false));
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

pub trait BufferType: Any + Send + Sync + 'static {
    fn any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any + Send + Sync + 'static> BufferType for T {
    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct Buffers {
    by_id_and_type: HashMap<(egui::Id, TypeId), Box<dyn BufferType>>,
}

impl Buffers {
    pub fn get_mut_or_insert_with<T: BufferType>(
        &mut self,
        id: egui::Id,
        default_provider: impl FnOnce() -> T,
    ) -> &mut T {
        let key = (id, TypeId::of::<T>());
        self.by_id_and_type
            .entry(key)
            .or_insert(Box::new(default_provider()))
            .as_mut()
            .any_mut()
            .downcast_mut::<T>()
            .expect("There must never be a type mismatch here")
    }

    pub fn get_mut_or_default<T: BufferType + Default>(&mut self, id: egui::Id) -> &mut T {
        self.get_mut_or_insert_with(id, || T::default())
    }

    pub fn clear<T: BufferType>(&mut self, id: egui::Id) {
        let key = (id, TypeId::of::<T>());
        self.by_id_and_type.remove(&key);
    }
}
