use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::{EntityEvent, Event},
    query::With,
    system::command::trigger,
    world::{CommandQueue, World},
};
use egui_dock::egui;
use egui_notify::Toasts;

use crate::ui::{
    actions::{EditorAction, PushQueue},
    core::Buffers,
    native_views::EditorViewState,
};

pub mod animation_clip_preview;
pub mod debugger;
pub mod event_sender;
pub mod event_track_editor;
pub mod fsm_editor;
pub mod fsm_picker;
pub mod graph_editor;
pub mod graph_picker;
pub mod inspector;
pub mod preview_hierarchy;
pub mod scene_picker;
pub mod scene_preview;
pub mod scene_preview_errors;

pub struct EditorWindowContext<'a> {
    pub window_entity: Entity,
    pub view_entity: Entity,
    pub notifications: &'a mut Toasts,
    pub command_queue: &'a mut CommandQueue,
    pub buffers: &'a mut Buffers,

    // Legacy stuff for backwards compat
    pub editor_actions: &'a mut PushQueue<EditorAction>,
}

impl EditorWindowContext<'_> {
    pub fn make_queue(&self) -> OwnedQueue {
        OwnedQueue {
            window_entity: self.window_entity,
            view_entity: self.view_entity,
            command_queue: CommandQueue::default(),
        }
    }

    pub fn consume_queue(&mut self, mut queue: OwnedQueue) {
        self.command_queue.append(&mut queue.command_queue);
    }
}

pub struct OwnedQueue {
    pub window_entity: Entity,
    pub view_entity: Entity,
    pub command_queue: CommandQueue,
}

impl OwnedQueue {
    pub fn trigger<'b>(&mut self, event: impl Event<Trigger<'b>: Default>) {
        self.command_queue.push(trigger(event));
    }

    pub fn trigger_window<'b>(
        &mut self,
        mut event: impl Event<Trigger<'b>: Default> + EntityEvent,
    ) {
        *event.event_target_mut() = self.window_entity;
        self.command_queue.push(trigger(event));
    }

    pub fn trigger_view<'b>(&mut self, mut event: impl Event<Trigger<'b>: Default> + EntityEvent) {
        *event.event_target_mut() = self.view_entity;
        self.command_queue.push(trigger(event));
    }
}

#[derive(Component)]
pub struct WindowState;

impl<'a> EditorWindowContext<'a> {
    pub fn trigger<'b>(&mut self, event: impl Event<Trigger<'b>: Default>) {
        self.command_queue.push(trigger(event));
    }

    pub fn get_window_state<'w, T: Component>(&self, world: &'w World) -> Option<&'w T> {
        let mut query = world.try_query_filtered::<&T, With<WindowState>>()?;
        query.get(world, self.window_entity).ok()
    }

    pub fn get_view_state<'w, T: Component>(&self, world: &'w World) -> Option<&'w T> {
        let mut query = world.try_query_filtered::<&T, With<EditorViewState>>()?;
        query.get(world, self.view_entity).ok()
    }
}

pub struct EditorWindowRegistrationContext {
    pub view: Entity,
    pub window: Entity,
}

pub trait NativeEditorWindowExtension: std::fmt::Debug + Send + Sync + 'static {
    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext);

    fn display_name(&self) -> String;

    #[allow(unused_variables)]
    fn init(&self, world: &mut World, ctx: &EditorWindowRegistrationContext) {}

    fn closeable(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct NativeEditorWindow {
    window: Box<dyn NativeEditorWindowExtension>,
    pub entity: Entity,
}

impl NativeEditorWindowExtension for NativeEditorWindow {
    fn init(&self, world: &mut World, ctx: &EditorWindowRegistrationContext) {
        self.window.init(world, ctx)
    }

    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        self.window.ui(ui, world, ctx);
    }

    fn display_name(&self) -> String {
        self.window.display_name()
    }

    fn closeable(&self) -> bool {
        self.window.closeable()
    }
}

impl NativeEditorWindow {
    pub fn create<T: NativeEditorWindowExtension>(
        world: &mut World,
        view_entity: Entity,
        ext: T,
    ) -> Self {
        let entity = world.spawn((WindowState,)).id();

        let ctx = EditorWindowRegistrationContext {
            window: entity,
            view: view_entity,
        };

        ext.init(world, &ctx);

        Self {
            window: Box::new(ext),
            entity,
        }
    }
}
